use std::process::ExitCode;
use std::str::FromStr;

use anyhow::Result;
use ticket_flow_core::clawhip::{
    BuildClawhipEventInput, ClawhipEventKind, ClawhipSendMode, SendClawhipEventInput,
    build_clawhip_event, send_clawhip_event,
};
use ticket_flow_core::{TicketId, TicketStore};

use crate::args::{ClawhipCommand, ClawhipEventArgs};

pub(crate) fn run(store: &TicketStore, command: ClawhipCommand) -> Result<ExitCode> {
    match command {
        ClawhipCommand::Event(args) => event(store, args),
    }
}

fn event(store: &TicketStore, args: ClawhipEventArgs) -> Result<ExitCode> {
    let id = TicketId::parse(&args.id)?;
    let ticket = store.get_ticket(&id)?;
    let kind = ClawhipEventKind::from_str(&args.kind)?;
    let event = build_clawhip_event(BuildClawhipEventInput {
        kind,
        ticket: &ticket,
        repo_path: std::env::var("TICKET_FLOW_REPO_PATH")
            .ok()
            .or_else(current_dir_text),
        worktree_path: std::env::var("TICKET_FLOW_WORKTREE_PATH").ok(),
        from_status: None,
        to_status: None,
        correlation_id: None,
    });
    if args.print || !args.send {
        println!("{}", serde_json::to_string_pretty(&event)?);
    }
    if args.send {
        let result = send_clawhip_event(SendClawhipEventInput {
            url: Some(args.url),
            event,
            mode: ClawhipSendMode::Strict,
            timeout_ms: args.timeout_ms,
        })?;
        eprintln!(
            "clawhip: sent {} status={}",
            kind,
            result.status.unwrap_or_default()
        );
    }
    Ok(ExitCode::SUCCESS)
}

fn current_dir_text() -> Option<String> {
    std::env::current_dir()
        .ok()
        .map(|path| path.display().to_string())
}
