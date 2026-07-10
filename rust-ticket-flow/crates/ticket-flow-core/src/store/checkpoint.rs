use serde_json::json;

use super::TicketStore;
use crate::clawhip::{ClawhipEventKind, EnvClawhipEmit, emit_ticket_event_from_env};
use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_AGENT_ACTION_AVAILABLE, TICKET_CHECKPOINTED};
use crate::model::{
    CheckpointPatch, CurrentState, LogEntry, NextAction, NextActionType, Ticket, TicketId,
    TicketStatus,
};
use crate::time::now_rfc3339;

impl TicketStore {
    pub fn checkpoint_ticket(&self, id: &TicketId, patch: CheckpointPatch) -> Result<Ticket> {
        if patch.is_empty() {
            return Err(TicketFlowError::EmptyCheckpoint);
        }
        let mut ticket = self.load_active(id)?;
        let now = now_rfc3339();
        let mut current = ticket.current.clone().unwrap_or_default();
        apply_checkpoint(&mut current, &patch);
        let next_action = current.next_action.clone();
        ticket.current = Some(current);
        ticket
            .log
            .push(checkpoint_log(&patch, next_action.clone(), &now));
        ticket.updated = now;
        if patch.blocker.is_some()
            || matches!(patch.next_action_type, Some(NextActionType::Blocked))
        {
            ticket.status = TicketStatus::Blocked;
        }
        let event = EventEnvelope::new(
            id.clone(),
            TICKET_CHECKPOINTED,
            serde_json::to_value(&patch)?,
        );
        self.commit_active(&ticket, event)?;
        emit_ticket_event_from_env(EnvClawhipEmit {
            kind: ClawhipEventKind::Checkpointed,
            ticket: &ticket,
            from_status: None,
            to_status: None,
        });
        self.emit_agent_action_if_needed(id, next_action, &ticket)?;
        if matches!(ticket.status, TicketStatus::Blocked) {
            emit_ticket_event_from_env(EnvClawhipEmit {
                kind: ClawhipEventKind::Blocked,
                ticket: &ticket,
                from_status: None,
                to_status: None,
            });
        }
        Ok(ticket)
    }

    fn emit_agent_action_if_needed(
        &self,
        id: &TicketId,
        next_action: Option<NextAction>,
        ticket: &Ticket,
    ) -> Result<()> {
        if matches!(
            next_action.and_then(|action| action.action_type),
            Some(NextActionType::AgentAction)
        ) {
            self.append_event(&EventEnvelope::new(
                id.clone(),
                TICKET_AGENT_ACTION_AVAILABLE,
                json!({ "next_action": ticket.current.as_ref().and_then(|c| c.next_action.clone()) }),
            ))?;
            emit_ticket_event_from_env(EnvClawhipEmit {
                kind: ClawhipEventKind::AgentActionAvailable,
                ticket,
                from_status: None,
                to_status: None,
            });
        }
        Ok(())
    }
}

fn checkpoint_log(patch: &CheckpointPatch, next_action: Option<NextAction>, now: &str) -> LogEntry {
    LogEntry {
        ts: now.to_owned(),
        action: Some("checkpoint".to_owned()),
        note: patch.note.clone(),
        phase: patch.phase.clone(),
        decision: patch.decision.clone(),
        evidence: patch.evidence.clone(),
        blocker: patch.blocker.clone(),
        next: patch.next.clone(),
        next_action,
        extra: Default::default(),
    }
}

fn apply_checkpoint(current: &mut CurrentState, patch: &CheckpointPatch) {
    if let Some(value) = &patch.phase {
        current.phase = Some(value.clone());
    }
    if let Some(value) = &patch.decision {
        current.decision = Some(value.clone());
    }
    if let Some(value) = &patch.evidence {
        current.evidence = Some(value.clone());
    }
    if let Some(value) = &patch.blocker {
        current.blocker = Some(value.clone());
    }
    if let Some(value) = &patch.next {
        current.next = Some(value.clone());
    }
    if let Some(value) = &patch.note {
        current.note = Some(value.clone());
    }
    let mut next_action = current.next_action.clone().unwrap_or_default();
    if let Some(value) = patch.next_action_type {
        next_action.action_type = Some(value);
    }
    if let Some(value) = &patch.next_command {
        next_action.command = Some(value.clone());
    }
    if let Some(value) = &patch.next_owner {
        next_action.owner = Some(value.clone());
    }
    if next_action.has_fields() {
        current.next_action = Some(next_action);
    }
}
