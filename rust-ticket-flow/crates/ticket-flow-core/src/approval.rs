use serde_json::json;
use uuid::Uuid;

use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_APPROVAL_REQUESTED, TICKET_APPROVAL_RESPONDED};
use crate::model::{ApprovalRecord, Ticket, TicketId};
use crate::store::TicketStore;
use crate::time::now_rfc3339;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApprovalRequest {
    pub owner: String,
    pub question: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
}

impl ApprovalDecision {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Approved => "approved",
            Self::Rejected => "rejected",
        }
    }
}

pub fn request_approval(
    store: &TicketStore,
    id: &TicketId,
    input: ApprovalRequest,
) -> Result<Ticket> {
    let mut ticket = store.load_active(id)?;
    let approval = ApprovalRecord {
        id: format!("A-{}", Uuid::now_v7()),
        owner: input.owner,
        question: input.question,
        status: "requested".to_owned(),
        requested_at: now_rfc3339(),
        outcome: None,
        responded_at: None,
    };
    let mut coordination = ticket.coordination.clone().unwrap_or_default();
    coordination.approvals.push(approval.clone());
    ticket.coordination = Some(coordination);
    ticket.updated = now_rfc3339();
    let event = EventEnvelope::new(id.clone(), TICKET_APPROVAL_REQUESTED, json!(approval));
    store.commit_active(&ticket, event)?;
    Ok(ticket)
}

pub fn respond_approval(
    store: &TicketStore,
    id: &TicketId,
    approval_id: &str,
    decision: ApprovalDecision,
) -> Result<Ticket> {
    let mut ticket = store.load_active(id)?;
    let mut coordination = ticket.coordination.clone().unwrap_or_default();
    let mut found = false;
    for approval in &mut coordination.approvals {
        if approval.id == approval_id {
            approval.status = "responded".to_owned();
            approval.outcome = Some(decision.as_str().to_owned());
            approval.responded_at = Some(now_rfc3339());
            found = true;
        }
    }
    if !found {
        return Err(TicketFlowError::ApprovalNotFound(approval_id.to_owned()));
    }
    ticket.coordination = Some(coordination);
    ticket.updated = now_rfc3339();
    let event = EventEnvelope::new(
        id.clone(),
        TICKET_APPROVAL_RESPONDED,
        json!({ "approval_id": approval_id, "outcome": decision.as_str() }),
    );
    store.commit_active(&ticket, event)?;
    Ok(ticket)
}
