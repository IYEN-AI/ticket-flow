use serde_json::json;

use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_EVIDENCE_ATTACHED};
use crate::model::{Artifact, EvidenceStatus, Ticket, TicketId};
use crate::store::TicketStore;
use crate::time::now_rfc3339;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceInput {
    pub artifact_type: String,
    pub value: String,
}

pub fn attach_evidence(store: &TicketStore, id: &TicketId, input: EvidenceInput) -> Result<Ticket> {
    if input.value.trim().is_empty() {
        return Err(TicketFlowError::EmptyEvidence);
    }
    let mut ticket = store.load_active(id)?;
    let artifact = Artifact::new(input.artifact_type, input.value, now_rfc3339());
    ticket.artifacts.push(artifact.clone());
    let mut evidence = ticket.evidence.clone().unwrap_or_default();
    evidence.artifacts.push(artifact);
    evidence.evidence_status = EvidenceStatus::Partial;
    ticket.evidence = Some(evidence);
    ticket.updated = now_rfc3339();
    let event = EventEnvelope::new(
        id.clone(),
        TICKET_EVIDENCE_ATTACHED,
        json!({ "evidence_status": EvidenceStatus::Partial }),
    );
    store.commit_active(&ticket, event)?;
    Ok(ticket)
}
