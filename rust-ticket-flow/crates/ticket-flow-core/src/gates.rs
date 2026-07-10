use serde::{Deserialize, Serialize};

use crate::model::{EvidenceStatus, NextActionType, Ticket, TicketStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateResult {
    pub gate: String,
    pub passed: bool,
    pub missing: Vec<String>,
    pub warnings: Vec<String>,
}

impl GateResult {
    fn new(gate: impl Into<String>, missing: Vec<String>, warnings: Vec<String>) -> Self {
        Self {
            gate: gate.into(),
            passed: missing.is_empty(),
            missing,
            warnings,
        }
    }
}

pub fn ready_gate(ticket: &Ticket) -> GateResult {
    let mut missing = Vec::new();
    if ticket.goal.trim().is_empty() {
        missing.push("goal".to_owned());
    }
    if ticket.acceptance.is_empty() {
        missing.push("acceptance".to_owned());
    }
    let next_action = ticket
        .current
        .as_ref()
        .and_then(|current| current.next_action.as_ref());
    if next_action.and_then(|action| action.action_type).is_none() {
        missing.push("current.next_action".to_owned());
    }
    GateResult::new("ready", missing, Vec::new())
}

pub fn blocked_gate(ticket: &Ticket) -> GateResult {
    let current = ticket.current.as_ref();
    let has_blocker = ticket.status == TicketStatus::Blocked
        || current
            .and_then(|state| state.blocker.as_deref())
            .is_some_and(|value| !value.trim().is_empty());
    if !has_blocker {
        return GateResult::new("blocked", Vec::new(), Vec::new());
    }
    let has_owner = current
        .and_then(|state| state.next_action.as_ref())
        .and_then(|action| action.owner.as_deref())
        .is_some_and(|owner| !owner.trim().is_empty());
    let missing = if has_owner {
        Vec::new()
    } else {
        vec!["current.next_action.owner".to_owned()]
    };
    GateResult::new("blocked", missing, Vec::new())
}

pub fn review_gate(ticket: &Ticket) -> GateResult {
    let has_artifact = !ticket.artifacts.is_empty()
        || ticket
            .evidence
            .as_ref()
            .is_some_and(|evidence| !evidence.artifacts.is_empty());
    let missing = if ticket.status == TicketStatus::Review && !has_artifact {
        vec!["evidence.artifacts".to_owned()]
    } else {
        Vec::new()
    };
    GateResult::new("review", missing, Vec::new())
}

pub fn done_gate(ticket: &Ticket) -> GateResult {
    let evidence_status = ticket
        .evidence
        .as_ref()
        .map(|evidence| &evidence.evidence_status);
    let has_evidence = matches!(
        evidence_status,
        Some(EvidenceStatus::Partial | EvidenceStatus::Sufficient)
    ) || !ticket.artifacts.is_empty();
    let mut missing = Vec::new();
    if ticket.acceptance.is_empty() {
        missing.push("acceptance".to_owned());
    }
    if !has_evidence {
        missing.push("evidence".to_owned());
    }
    GateResult::new("done", missing, Vec::new())
}

pub fn next_action_gate(ticket: &Ticket, expected: NextActionType) -> GateResult {
    let actual = ticket
        .current
        .as_ref()
        .and_then(|state| state.next_action.as_ref())
        .and_then(|action| action.action_type);
    let missing = if actual == Some(expected) {
        Vec::new()
    } else {
        vec!["current.next_action.type".to_owned()]
    };
    GateResult::new("next_action", missing, Vec::new())
}
