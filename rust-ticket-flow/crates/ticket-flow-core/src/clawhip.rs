use std::fmt::{Display, Formatter};
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::error::{Result, TicketFlowError};
use crate::model::{NextActionType, Ticket, TicketStatus};

const DEFAULT_CLAWHIP_URL: &str = "http://127.0.0.1:25294";
const DEFAULT_TIMEOUT_MS: u64 = 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClawhipEventKind {
    #[serde(rename = "ticket.created")]
    Created,
    #[serde(rename = "ticket.status_changed")]
    StatusChanged,
    #[serde(rename = "ticket.checkpointed")]
    Checkpointed,
    #[serde(rename = "ticket.agent_action_available")]
    AgentActionAvailable,
    #[serde(rename = "ticket.blocked")]
    Blocked,
    #[serde(rename = "ticket.review_ready")]
    ReviewReady,
}

impl ClawhipEventKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Created => "ticket.created",
            Self::StatusChanged => "ticket.status_changed",
            Self::Checkpointed => "ticket.checkpointed",
            Self::AgentActionAvailable => "ticket.agent_action_available",
            Self::Blocked => "ticket.blocked",
            Self::ReviewReady => "ticket.review_ready",
        }
    }
}

impl Display for ClawhipEventKind {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for ClawhipEventKind {
    type Err = TicketFlowError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "ticket.created" => Ok(Self::Created),
            "ticket.status_changed" => Ok(Self::StatusChanged),
            "ticket.checkpointed" => Ok(Self::Checkpointed),
            "ticket.agent_action_available" => Ok(Self::AgentActionAvailable),
            "ticket.blocked" => Ok(Self::Blocked),
            "ticket.review_ready" => Ok(Self::ReviewReady),
            other => Err(TicketFlowError::InvalidClawhipEventKind(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClawhipTicketEvent {
    #[serde(rename = "type")]
    pub event_type: ClawhipEventKind,
    pub payload: ClawhipTicketPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClawhipTicketPayload {
    pub provider: String,
    pub event: ClawhipEventKind,
    pub ticket_id: String,
    pub title: String,
    pub status: TicketStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ticket_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_status: Option<TicketStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_status: Option<TicketStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocker: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action_type: Option<NextActionType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_action_owner: Option<String>,
    pub summary: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub worktree_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_name: Option<String>,
    pub event_timestamp: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BuildClawhipEventInput<'a> {
    pub kind: ClawhipEventKind,
    pub ticket: &'a Ticket,
    pub repo_path: Option<String>,
    pub worktree_path: Option<String>,
    pub from_status: Option<TicketStatus>,
    pub to_status: Option<TicketStatus>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClawhipSendMode {
    BestEffort,
    Strict,
}

#[derive(Debug, Clone)]
pub struct SendClawhipEventInput {
    pub url: Option<String>,
    pub event: ClawhipTicketEvent,
    pub mode: ClawhipSendMode,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClawhipSendResult {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

pub fn build_clawhip_event(input: BuildClawhipEventInput<'_>) -> ClawhipTicketEvent {
    let current = input.ticket.current.as_ref();
    let next_action = current.and_then(|state| state.next_action.as_ref());
    let repo_path = resolve_text(input.repo_path);
    let worktree_path = resolve_text(input.worktree_path).or_else(|| repo_path.clone());
    ClawhipTicketEvent {
        event_type: input.kind,
        payload: ClawhipTicketPayload {
            provider: "ticket-flow".to_owned(),
            event: input.kind,
            ticket_id: input.ticket.id.to_string(),
            title: input.ticket.title.clone(),
            status: input.ticket.status,
            priority: resolve_text(Some(input.ticket.priority.clone())),
            ticket_type: resolve_text(Some(input.ticket.ticket_type.clone())),
            assignee: resolve_text(Some(input.ticket.assignee.clone())),
            from_status: input.from_status,
            to_status: input.to_status,
            phase: current.and_then(|state| resolve_text(state.phase.clone())),
            decision: current.and_then(|state| resolve_text(state.decision.clone())),
            blocker: current.and_then(|state| resolve_text(state.blocker.clone())),
            next: current.and_then(|state| resolve_text(state.next.clone())),
            next_action_type: next_action.and_then(|action| action.action_type),
            next_action_command: next_action
                .and_then(|action| resolve_text(action.command.clone())),
            next_action_owner: next_action.and_then(|action| resolve_text(action.owner.clone())),
            summary: format!(
                "{} [{}] {}",
                input.ticket.id, input.ticket.status, input.ticket.title
            ),
            question_summary: question_summary(input.kind, input.ticket),
            repo_name: repo_path.as_deref().and_then(repo_name),
            repo_path,
            worktree_path,
            event_timestamp: input.ticket.updated.clone(),
            correlation_id: resolve_text(input.correlation_id),
        },
    }
}

pub fn send_clawhip_event(input: SendClawhipEventInput) -> Result<ClawhipSendResult> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(input.timeout_ms))
        .build()?;
    let response = client
        .post(event_endpoint(
            input.url.as_deref().unwrap_or(DEFAULT_CLAWHIP_URL),
        ))
        .json(&input.event)
        .send();
    match response {
        Ok(value) => Ok(ClawhipSendResult {
            ok: true,
            status: Some(value.status().as_u16()),
            error: None,
        }),
        Err(error) if input.mode == ClawhipSendMode::BestEffort => Ok(ClawhipSendResult {
            ok: false,
            status: None,
            error: Some(error.to_string()),
        }),
        Err(error) => Err(error.into()),
    }
}

pub struct EnvClawhipEmit<'a> {
    pub kind: ClawhipEventKind,
    pub ticket: &'a Ticket,
    pub from_status: Option<TicketStatus>,
    pub to_status: Option<TicketStatus>,
}

pub fn emit_ticket_event_from_env(input: EnvClawhipEmit<'_>) {
    if !clawhip_enabled() {
        return;
    }
    let event = build_clawhip_event(BuildClawhipEventInput {
        kind: input.kind,
        ticket: input.ticket,
        repo_path: std::env::var("TICKET_FLOW_REPO_PATH").ok(),
        worktree_path: std::env::var("TICKET_FLOW_WORKTREE_PATH").ok(),
        from_status: input.from_status,
        to_status: input.to_status,
        correlation_id: None,
    });
    let _ = send_clawhip_event(SendClawhipEventInput {
        url: std::env::var("TICKET_FLOW_CLAWHIP_URL").ok(),
        event,
        mode: ClawhipSendMode::BestEffort,
        timeout_ms: clawhip_timeout_ms(),
    });
}

fn event_endpoint(base_url: &str) -> String {
    let trimmed = base_url.trim_end_matches('/');
    if trimmed.ends_with("/event") {
        trimmed.to_owned()
    } else {
        format!("{trimmed}/event")
    }
}

fn resolve_text(value: Option<String>) -> Option<String> {
    value.and_then(|text| {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_owned())
        }
    })
}

fn question_summary(kind: ClawhipEventKind, ticket: &Ticket) -> Option<String> {
    match kind {
        ClawhipEventKind::Blocked | ClawhipEventKind::ReviewReady => ticket
            .current
            .as_ref()
            .and_then(|state| state.blocker.clone().or_else(|| state.next.clone()))
            .or_else(|| Some(ticket.title.clone())),
        ClawhipEventKind::Created
        | ClawhipEventKind::StatusChanged
        | ClawhipEventKind::Checkpointed
        | ClawhipEventKind::AgentActionAvailable => None,
    }
}

fn repo_name(path: &str) -> Option<String> {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .map(ToOwned::to_owned)
}

fn clawhip_enabled() -> bool {
    std::env::var("TICKET_FLOW_CLAWHIP")
        .map(|value| matches!(value.as_str(), "1" | "true" | "yes"))
        .unwrap_or(false)
}

fn clawhip_timeout_ms() -> u64 {
    std::env::var("TICKET_FLOW_CLAWHIP_TIMEOUT_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(DEFAULT_TIMEOUT_MS)
}
