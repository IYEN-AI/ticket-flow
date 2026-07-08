use anyhow::{Result, bail};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};
use ticket_flow_core::{
    AddLogInput, CheckpointPatch, CreateTicket, LinkKind, LinkTicketInput, StatusPatch, TicketId,
    TicketStatus, TicketStore, agent_queue,
};

pub(crate) fn tool_definitions() -> Value {
    json!([
        {"name":"ticket_create","description":"Create an agent ticket.","inputSchema":{"type":"object"}},
        {"name":"ticket_list","description":"List active agent tickets.","inputSchema":{"type":"object"}},
        {"name":"ticket_get","description":"Read an active or archived ticket.","inputSchema":{"type":"object"}},
        {"name":"ticket_import","description":"Import a source ticket store into ticket-flow.","inputSchema":{"type":"object"}},
        {"name":"ticket_update_status","description":"Apply a ticket status transition.","inputSchema":{"type":"object"}},
        {"name":"ticket_link","description":"Attach an external reference to a ticket.","inputSchema":{"type":"object"}},
        {"name":"ticket_add_log","description":"Append a note log entry to a ticket.","inputSchema":{"type":"object"}},
        {"name":"ticket_checkpoint","description":"Write the current checkpoint payload.","inputSchema":{"type":"object"}},
        {"name":"ticket_agent_actions","description":"List tickets with agent_action next actions.","inputSchema":{"type":"object"}}
    ])
}

pub(crate) fn call(params: &Value) -> Result<Value> {
    let call = serde_json::from_value::<ToolCallParams>(params.clone())?;
    let store = TicketStore::new(ticket_flow_core::StorePaths::from_env_or_default()?);
    match call.name.as_str() {
        "ticket_create" => ticket_create(&store, call.arguments),
        "ticket_list" => ticket_list(&store, call.arguments),
        "ticket_get" => ticket_get(&store, call.arguments),
        "ticket_import" => ticket_import(&store, call.arguments),
        "ticket_update_status" => ticket_update_status(&store, call.arguments),
        "ticket_link" => ticket_link(&store, call.arguments),
        "ticket_add_log" => ticket_add_log(&store, call.arguments),
        "ticket_checkpoint" => ticket_checkpoint(&store, call.arguments),
        "ticket_agent_actions" => text_result(&agent_queue(&store)?),
        other => bail!("unknown tool {other}"),
    }
}

#[derive(Deserialize)]
struct ToolCallParams {
    name: String,
    #[serde(default = "empty_arguments")]
    arguments: Value,
}

#[derive(Deserialize)]
struct CreateToolArgs {
    title: String,
    #[serde(default = "default_ticket_type", rename = "type")]
    ticket_type: String,
    #[serde(default = "default_priority")]
    priority: String,
    #[serde(default)]
    goal: String,
    #[serde(default = "default_assignee")]
    assignee: String,
    #[serde(default)]
    acceptance: Vec<String>,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    parent: Option<String>,
    #[serde(default)]
    source: Option<String>,
}

#[derive(Deserialize)]
struct IdToolArgs {
    id: String,
}

#[derive(Deserialize)]
struct ListToolArgs {
    status: Option<TicketStatus>,
}

#[derive(Deserialize)]
struct ImportToolArgs {
    #[serde(rename = "sourceRoot")]
    source_root: String,
}

#[derive(Deserialize)]
struct StatusToolArgs {
    id: String,
    status: TicketStatus,
    artifact: Option<String>,
    evidence: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct LinkToolArgs {
    id: String,
    kind: LinkKind,
    value: String,
}

#[derive(Deserialize)]
struct AddLogToolArgs {
    id: String,
    note: String,
}

#[derive(Deserialize)]
struct CheckpointToolArgs {
    id: String,
    phase: Option<String>,
    decision: Option<String>,
    evidence: Option<String>,
    blocker: Option<String>,
    next: Option<String>,
    note: Option<String>,
    #[serde(rename = "nextType")]
    next_action_type: Option<ticket_flow_core::NextActionType>,
    #[serde(rename = "nextCommand")]
    next_command: Option<String>,
    #[serde(rename = "nextOwner")]
    next_owner: Option<String>,
}

fn ticket_create(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<CreateToolArgs>(value)?;
    if args.title.trim().is_empty() {
        bail!("title must not be empty");
    }
    let mut input = CreateTicket::new(args.title);
    input.ticket_type = args.ticket_type;
    input.priority = args.priority;
    input.goal = args.goal;
    input.assignee = args.assignee;
    input.acceptance = args.acceptance;
    input.tags = args.tags;
    input.parent = args.parent;
    input.source = source_value(args.source)?;
    text_result(&store.create_ticket(input)?)
}

fn ticket_list(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<ListToolArgs>(value)?;
    let tickets = store
        .list_tickets()?
        .into_iter()
        .filter(|ticket| args.status.is_none_or(|status| ticket.status == status))
        .collect::<Vec<_>>();
    text_result(&tickets)
}

fn ticket_get(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<IdToolArgs>(value)?;
    text_result(&store.get_ticket(&TicketId::parse(&args.id)?)?)
}

fn ticket_import(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<ImportToolArgs>(value)?;
    text_result(&store.import_store(args.source_root)?)
}

fn ticket_update_status(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<StatusToolArgs>(value)?;
    let id = TicketId::parse(&args.id)?;
    text_result(&store.update_status(
        &id,
        StatusPatch {
            status: args.status,
            artifact: args.artifact,
            evidence: args.evidence,
            note: args.note,
        },
    )?)
}

fn ticket_link(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<LinkToolArgs>(value)?;
    let id = TicketId::parse(&args.id)?;
    text_result(&store.link_ticket(
        &id,
        LinkTicketInput {
            kind: args.kind,
            value: args.value,
        },
    )?)
}

fn ticket_add_log(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<AddLogToolArgs>(value)?;
    let id = TicketId::parse(&args.id)?;
    text_result(&store.add_log(&id, AddLogInput { note: args.note })?)
}

fn ticket_checkpoint(store: &TicketStore, value: Value) -> Result<Value> {
    let args = serde_json::from_value::<CheckpointToolArgs>(value)?;
    let id = TicketId::parse(&args.id)?;
    text_result(&store.checkpoint_ticket(
        &id,
        CheckpointPatch {
            phase: args.phase,
            decision: args.decision,
            evidence: args.evidence,
            blocker: args.blocker,
            next: args.next,
            note: args.note,
            next_action_type: args.next_action_type,
            next_command: args.next_command,
            next_owner: args.next_owner,
        },
    )?)
}

fn text_result(value: &impl Serialize) -> Result<Value> {
    Ok(json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(value)? }] }))
}

fn source_value(value: Option<String>) -> Result<Option<Value>> {
    let Some(source) = value else {
        return Ok(None);
    };
    let Some((kind, reference)) = source.split_once(':') else {
        bail!("invalid source {source}");
    };
    if kind.trim().is_empty() || reference.trim().is_empty() {
        bail!("invalid source {source}");
    }
    Ok(Some(json!({ "type": kind, "ref": reference })))
}

fn empty_arguments() -> Value {
    json!({})
}

fn default_ticket_type() -> String {
    "chore".to_owned()
}

fn default_priority() -> String {
    "medium".to_owned()
}

fn default_assignee() -> String {
    "iyen".to_owned()
}
