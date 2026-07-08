use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{NextAction, Ticket, TicketId};
use crate::store::TicketStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextAudience {
    AgentExecution,
    OwnerReview,
    ReleaseReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextPack {
    pub audience: ContextAudience,
    pub ticket_id: TicketId,
    pub goal: String,
    pub current_state: Option<String>,
    pub next_action: Option<NextAction>,
    pub constraints: Vec<String>,
    pub acceptance: Vec<String>,
    pub blockers: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub risk_flags: Vec<String>,
}

pub fn build_context_pack(
    store: &TicketStore,
    id: &TicketId,
    audience: ContextAudience,
) -> Result<ContextPack> {
    let ticket = store.get_ticket(id)?;
    Ok(context_pack_from_ticket(&ticket, audience))
}

fn context_pack_from_ticket(ticket: &Ticket, audience: ContextAudience) -> ContextPack {
    let constraints = ticket
        .shared
        .as_ref()
        .map(|shared| shared.constraints.clone())
        .unwrap_or_default();
    let blockers = ticket
        .coordination
        .as_ref()
        .map(|coordination| coordination.blockers.clone())
        .unwrap_or_default();
    let evidence_refs = ticket
        .evidence
        .as_ref()
        .map(|evidence| {
            evidence
                .artifacts
                .iter()
                .map(|artifact| artifact.value.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    ContextPack {
        audience,
        ticket_id: ticket.id.clone(),
        goal: ticket.goal.clone(),
        current_state: ticket
            .current
            .as_ref()
            .and_then(|current| current.phase.clone().or_else(|| current.decision.clone())),
        next_action: ticket
            .current
            .as_ref()
            .and_then(|current| current.next_action.clone()),
        constraints,
        acceptance: ticket.acceptance.clone(),
        blockers,
        evidence_refs,
        risk_flags: risk_flags(ticket),
    }
}

fn risk_flags(ticket: &Ticket) -> Vec<String> {
    let mut flags = Vec::new();
    if ticket.acceptance.is_empty() {
        flags.push("missing_acceptance".to_owned());
    }
    if ticket
        .current
        .as_ref()
        .and_then(|current| current.blocker.as_deref())
        .is_some_and(|blocker| !blocker.trim().is_empty())
    {
        flags.push("blocked".to_owned());
    }
    if ticket
        .evidence
        .as_ref()
        .is_none_or(|evidence| evidence.artifacts.is_empty())
    {
        flags.push("missing_evidence".to_owned());
    }
    flags
}
