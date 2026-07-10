use std::process::ExitCode;

use anyhow::Result;
use serde_json::json;
use ticket_flow_core::{
    ApprovalRequest, CheckpointPatch, CreateTicket, EvidenceInput, HandoffAck, HandoffRequest,
    StatusPatch, TicketId, TicketStore, ack_handoff, attach_evidence, blocked_gate, board,
    build_context_pack, coordination, ready_gate, request_approval, request_handoff,
    respond_approval, review,
};

use crate::args::{
    ApprovalCommand, Cli, Command, ContextPackArgs, CreateArgs, EvidenceCommand, HandoffCommand,
    IdArgs, ImportArgs, ListArgs, OutputFormat, ViewCommand,
};
use crate::support::{
    parse_approval_decision, parse_audience, parse_optional_next_action, parse_source,
    parse_status, paths, print_json_or_pretty, print_subject,
};

pub fn run(cli: Cli) -> Result<ExitCode> {
    let format = cli.format;
    let store = TicketStore::new(paths(cli.home)?);
    match cli.command {
        Command::Create(args) => create(&store, args),
        Command::Show(args) => show(&store, args, format),
        Command::List(args) => list(&store, args, format),
        Command::Status(args) => {
            let id = TicketId::parse(&args.id)?;
            let status = parse_status(&args.status)?;
            let ticket = store.update_status(
                &id,
                StatusPatch {
                    status,
                    artifact: args.artifact,
                    evidence: args.evidence,
                    note: args.note,
                },
            )?;
            print_subject(
                format,
                &ticket,
                &format!("{}: status {}", ticket.id, ticket.status),
            )
        }
        Command::Checkpoint(args) => {
            let id = TicketId::parse(&args.id)?;
            let patch = CheckpointPatch {
                phase: args.phase,
                decision: args.decision,
                evidence: args.evidence,
                blocker: args.blocker,
                next: args.next,
                note: args.note,
                next_action_type: parse_optional_next_action(args.next_type)?,
                next_command: args.next_command,
                next_owner: args.next_owner,
            };
            let ticket = store.checkpoint_ticket(&id, patch)?;
            print_subject(format, &ticket, &format!("{id}: checkpoint logged"))
        }
        Command::Import(args) => import_store(&store, args, format),
        Command::ReadyCheck(args) => ready_check(&store, args, format),
        Command::Evidence { command } => evidence(&store, command, format),
        Command::Handoff { command } => handoff(&store, command, format),
        Command::Approval { command } => approval(&store, command, format),
        Command::Clawhip { command } => crate::clawhip_cmd::run(&store, command),
        Command::View { command } => view(&store, command, format),
        Command::ContextPack(args) => context_pack(&store, args, format),
    }
}

fn import_store(store: &TicketStore, args: ImportArgs, format: OutputFormat) -> Result<ExitCode> {
    let summary = store.import_store(args.source)?;
    print_subject(
        format,
        &summary,
        &format!(
            "imported {} ticket(s): active={} archived={}",
            summary.imported, summary.active, summary.archived
        ),
    )
}

fn create(store: &TicketStore, args: CreateArgs) -> Result<ExitCode> {
    let mut input = CreateTicket::new(args.title);
    input.ticket_type = args.ticket_type;
    input.priority = args.priority;
    input.goal = args.goal;
    input.parent = args.parent;
    input.assignee = args.assignee;
    input.acceptance = args.acceptance;
    input.tags = args.tags;
    input.source = parse_source(args.source)?;
    let ticket = store.create_ticket(input)?;
    println!("{}", ticket.id);
    Ok(ExitCode::SUCCESS)
}

fn show(store: &TicketStore, args: IdArgs, format: OutputFormat) -> Result<ExitCode> {
    let id = TicketId::parse(&args.id)?;
    let ticket = store.get_ticket(&id)?;
    print_subject(format, &ticket, &format!("{} {}", ticket.id, ticket.title))
}

fn list(store: &TicketStore, args: ListArgs, format: OutputFormat) -> Result<ExitCode> {
    let status = match args.status {
        Some(value) => Some(parse_status(&value)?),
        None => None,
    };
    let tickets = store
        .list_tickets()?
        .into_iter()
        .filter(|ticket| status.is_none_or(|wanted| ticket.status == wanted))
        .collect::<Vec<_>>();
    print_subject(format, &tickets, &format!("{} ticket(s)", tickets.len()))
}

fn ready_check(store: &TicketStore, args: IdArgs, format: OutputFormat) -> Result<ExitCode> {
    let id = TicketId::parse(&args.id)?;
    let ticket = store.get_ticket(&id)?;
    let ready = ready_gate(&ticket);
    let blocked = blocked_gate(&ticket);
    let payload = json!({
        "passed": ready.passed && blocked.passed,
        "gates": [ready, blocked],
    });
    print_json_or_pretty(format, &payload, "ready check")?;
    if payload["passed"].as_bool() == Some(true) {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::FAILURE)
    }
}

fn evidence(
    store: &TicketStore,
    command: EvidenceCommand,
    format: OutputFormat,
) -> Result<ExitCode> {
    match command {
        EvidenceCommand::Attach(args) => {
            let id = TicketId::parse(&args.id)?;
            let ticket = attach_evidence(
                store,
                &id,
                EvidenceInput {
                    artifact_type: args.artifact_type,
                    value: args.value,
                },
            )?;
            print_subject(format, &ticket, &format!("{id}: evidence attached"))
        }
    }
}

fn handoff(store: &TicketStore, command: HandoffCommand, format: OutputFormat) -> Result<ExitCode> {
    match command {
        HandoffCommand::Request(args) => {
            let id = TicketId::parse(&args.id)?;
            let ticket = request_handoff(
                store,
                &id,
                HandoffRequest {
                    from: args.from,
                    to: args.to,
                    reason: args.reason,
                },
            )?;
            print_subject(format, &ticket, &format!("{id}: handoff requested"))
        }
        HandoffCommand::Ack(args) => {
            let id = TicketId::parse(&args.id)?;
            let ticket = ack_handoff(
                store,
                &id,
                HandoffAck {
                    handoff_id: args.handoff_id,
                },
            )?;
            print_subject(format, &ticket, &format!("{id}: handoff acknowledged"))
        }
    }
}

fn approval(
    store: &TicketStore,
    command: ApprovalCommand,
    format: OutputFormat,
) -> Result<ExitCode> {
    match command {
        ApprovalCommand::Request(args) => {
            let id = TicketId::parse(&args.id)?;
            let ticket = request_approval(
                store,
                &id,
                ApprovalRequest {
                    owner: args.owner,
                    question: args.question,
                },
            )?;
            print_subject(format, &ticket, &format!("{id}: approval requested"))
        }
        ApprovalCommand::Respond(args) => {
            let id = TicketId::parse(&args.id)?;
            let decision = parse_approval_decision(&args.decision)?;
            let ticket = respond_approval(store, &id, &args.approval_id, decision)?;
            print_subject(format, &ticket, &format!("{id}: approval responded"))
        }
    }
}

fn view(store: &TicketStore, command: ViewCommand, format: OutputFormat) -> Result<ExitCode> {
    match command {
        ViewCommand::AgentQueue => print_subject(
            format,
            &ticket_flow_core::views::agent_queue(store)?,
            "agent queue",
        ),
        ViewCommand::Board => print_subject(format, &board(store)?, "board"),
        ViewCommand::Review => print_subject(format, &review(store)?, "review"),
        ViewCommand::Coordination => print_subject(format, &coordination(store)?, "coordination"),
    }
}

fn context_pack(
    store: &TicketStore,
    args: ContextPackArgs,
    format: OutputFormat,
) -> Result<ExitCode> {
    let id = TicketId::parse(&args.id)?;
    let audience = parse_audience(&args.audience)?;
    let pack = build_context_pack(store, &id, audience)?;
    print_subject(format, &pack, "context pack")
}
