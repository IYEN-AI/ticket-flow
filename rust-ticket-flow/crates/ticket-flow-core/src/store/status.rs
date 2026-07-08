use std::collections::BTreeMap;

use serde_json::json;

use super::TicketStore;
use crate::clawhip::{ClawhipEventKind, EnvClawhipEmit, emit_ticket_event_from_env};
use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_STATUS_CHANGED};
use crate::model::{Artifact, LogEntry, Ticket, TicketId, TicketStatus};
use crate::time::now_rfc3339;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPatch {
    pub status: TicketStatus,
    pub artifact: Option<String>,
    pub evidence: Option<String>,
    pub note: Option<String>,
}

impl TicketStore {
    pub fn update_status(&self, id: &TicketId, patch: StatusPatch) -> Result<Ticket> {
        let mut ticket = self.load_active(id)?;
        let from = ticket.status;
        if !can_transition(from, patch.status) {
            return Err(TicketFlowError::InvalidTransition {
                from,
                to: patch.status,
            });
        }
        if patch.status == TicketStatus::Review
            && patch.artifact.is_none()
            && ticket.artifacts.is_empty()
        {
            return Err(TicketFlowError::ReviewArtifactRequired);
        }

        let now = now_rfc3339();
        ticket.status = patch.status;
        ticket.updated = now.clone();
        if patch.status == TicketStatus::Done {
            ticket.closed = Some(now.clone());
        }
        append_status_artifacts(&mut ticket, &patch, &now);
        ticket
            .log
            .push(status_log(from, patch.status, patch.note.clone(), now));

        let event = EventEnvelope::new(
            id.clone(),
            TICKET_STATUS_CHANGED,
            json!({ "from": from, "to": patch.status, "note": patch.note }),
        );
        if patch.status == TicketStatus::Done {
            self.archive_ticket(&ticket, event)?;
        } else {
            self.commit_active(&ticket, event)?;
        }
        emit_ticket_event_from_env(EnvClawhipEmit {
            kind: ClawhipEventKind::StatusChanged,
            ticket: &ticket,
            from_status: Some(from),
            to_status: Some(patch.status),
        });
        Ok(ticket)
    }
}

fn append_status_artifacts(ticket: &mut Ticket, patch: &StatusPatch, now: &str) {
    if let Some(value) = &patch.artifact {
        ticket
            .artifacts
            .push(Artifact::new("artifact", value.clone(), now.to_owned()));
    }
    if let Some(value) = &patch.evidence {
        ticket
            .artifacts
            .push(Artifact::new("evidence", value.clone(), now.to_owned()));
    }
}

fn status_log(from: TicketStatus, to: TicketStatus, note: Option<String>, now: String) -> LogEntry {
    LogEntry {
        ts: now,
        action: Some(format!("{from} -> {to}")),
        note,
        phase: None,
        decision: None,
        evidence: None,
        blocker: None,
        next: None,
        next_action: None,
        extra: BTreeMap::new(),
    }
}

const fn can_transition(from: TicketStatus, to: TicketStatus) -> bool {
    match from {
        TicketStatus::Open => matches!(to, TicketStatus::Doing | TicketStatus::Blocked),
        TicketStatus::Doing => matches!(
            to,
            TicketStatus::Review | TicketStatus::Blocked | TicketStatus::Open
        ),
        TicketStatus::Review => matches!(to, TicketStatus::Done | TicketStatus::Open),
        TicketStatus::Blocked => matches!(to, TicketStatus::Doing | TicketStatus::Open),
        TicketStatus::Done => false,
    }
}
