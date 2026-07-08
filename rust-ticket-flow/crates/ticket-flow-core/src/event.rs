use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::model::TicketId;
use crate::time::now_rfc3339;

pub const TICKET_AGENT_ACTION_AVAILABLE: &str = "ticket.agent_action_available";
pub const TICKET_APPROVAL_REQUESTED: &str = "ticket.approval_requested";
pub const TICKET_APPROVAL_RESPONDED: &str = "ticket.approval_responded";
pub const TICKET_CHECKPOINTED: &str = "ticket.checkpointed";
pub const TICKET_CREATED: &str = "ticket.created";
pub const TICKET_EVIDENCE_ATTACHED: &str = "ticket.evidence_attached";
pub const TICKET_HANDOFF_ACKED: &str = "ticket.handoff_acked";
pub const TICKET_HANDOFF_REQUESTED: &str = "ticket.handoff_requested";
pub const TICKET_IMPORTED: &str = "ticket.imported";
pub const TICKET_LINKED: &str = "ticket.linked";
pub const TICKET_LOG_ADDED: &str = "ticket.log_added";
pub const TICKET_STATUS_CHANGED: &str = "ticket.status_changed";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: String,
    pub ticket_id: TicketId,
    #[serde(rename = "type")]
    pub event_type: String,
    pub at: String,
    pub actor: String,
    pub payload: Value,
}

impl EventEnvelope {
    pub fn new(ticket_id: TicketId, event_type: impl Into<String>, payload: Value) -> Self {
        Self {
            id: Uuid::now_v7().to_string(),
            ticket_id,
            event_type: event_type.into(),
            at: now_rfc3339(),
            actor: "ticket-flow".to_owned(),
            payload,
        }
    }
}
