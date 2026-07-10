use serde_json::json;
use uuid::Uuid;

use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_HANDOFF_ACKED, TICKET_HANDOFF_REQUESTED};
use crate::model::{HandoffRecord, Ticket, TicketId};
use crate::store::TicketStore;
use crate::time::now_rfc3339;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffRequest {
    pub from: String,
    pub to: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffAck {
    pub handoff_id: String,
}

pub fn request_handoff(
    store: &TicketStore,
    id: &TicketId,
    input: HandoffRequest,
) -> Result<Ticket> {
    let mut ticket = store.load_active(id)?;
    let handoff = HandoffRecord {
        id: format!("H-{}", Uuid::now_v7()),
        from: input.from,
        to: input.to,
        reason: input.reason,
        status: "requested".to_owned(),
        requested_at: now_rfc3339(),
        acknowledged_at: None,
    };
    let mut coordination = ticket.coordination.clone().unwrap_or_default();
    coordination.handoffs.push(handoff.clone());
    ticket.coordination = Some(coordination);
    ticket.updated = now_rfc3339();
    let event = EventEnvelope::new(id.clone(), TICKET_HANDOFF_REQUESTED, json!(handoff));
    store.commit_active(&ticket, event)?;
    Ok(ticket)
}

pub fn ack_handoff(store: &TicketStore, id: &TicketId, input: HandoffAck) -> Result<Ticket> {
    let mut ticket = store.load_active(id)?;
    let mut coordination = ticket.coordination.clone().unwrap_or_default();
    let mut found = false;
    for handoff in &mut coordination.handoffs {
        if handoff.id == input.handoff_id {
            handoff.status = "acknowledged".to_owned();
            handoff.acknowledged_at = Some(now_rfc3339());
            found = true;
        }
    }
    if !found {
        return Err(TicketFlowError::HandoffNotFound(input.handoff_id));
    }
    ticket.coordination = Some(coordination);
    ticket.updated = now_rfc3339();
    let event = EventEnvelope::new(
        id.clone(),
        TICKET_HANDOFF_ACKED,
        json!({ "handoff_id": input.handoff_id }),
    );
    store.commit_active(&ticket, event)?;
    Ok(ticket)
}
