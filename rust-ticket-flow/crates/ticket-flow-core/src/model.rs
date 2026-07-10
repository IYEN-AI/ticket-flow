use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::error::TicketFlowError;

mod artifact;
mod sections;
mod ticket;

pub use artifact::{Artifact, LinkMap, LogEntry};
pub use sections::{
    ApprovalRecord, CoordinationSection, EvidenceSection, EvidenceStatus, ExecutionSection,
    HandoffRecord, LocalSection, MemorySection, SharedSection,
};
pub use ticket::{CreateTicket, Ticket, TicketIndex, TicketSummary};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct TicketId(String);

impl TicketId {
    pub fn parse(value: &str) -> crate::Result<Self> {
        if is_ticket_id(value) {
            Ok(Self(value.to_owned()))
        } else {
            Err(TicketFlowError::InvalidTicketId(value.to_owned()))
        }
    }

    pub fn from_parts(day: &str, sequence: u16) -> crate::Result<Self> {
        let value = format!("T-{day}-{sequence:03}");
        Self::parse(&value)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for TicketId {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TicketId {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(serde::de::Error::custom)
    }
}

fn is_ticket_id(value: &str) -> bool {
    let mut parts = value.split('-');
    let valid_day = parts.next().is_some_and(|day| day == "T")
        && parts
            .next()
            .is_some_and(|day| day.len() == 8 && day.chars().all(|c| c.is_ascii_digit()));
    let valid_sequence = parts
        .next()
        .is_some_and(|seq| seq.len() == 3 && seq.chars().all(|c| c.is_ascii_digit()));
    valid_day && valid_sequence && parts.next().is_none()
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    #[default]
    Open,
    Doing,
    Review,
    Blocked,
    Done,
}

impl Display for TicketStatus {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            Self::Open => "open",
            Self::Doing => "doing",
            Self::Review => "review",
            Self::Blocked => "blocked",
            Self::Done => "done",
        };
        formatter.write_str(value)
    }
}

impl FromStr for TicketStatus {
    type Err = TicketFlowError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "open" => Ok(Self::Open),
            "doing" => Ok(Self::Doing),
            "review" => Ok(Self::Review),
            "blocked" => Ok(Self::Blocked),
            "done" => Ok(Self::Done),
            other => Err(TicketFlowError::InvalidStatus(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NextActionType {
    AgentAction,
    OwnerGate,
    ReleaseGate,
    Blocked,
}

impl FromStr for NextActionType {
    type Err = TicketFlowError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "agent_action" => Ok(Self::AgentAction),
            "owner_gate" => Ok(Self::OwnerGate),
            "release_gate" => Ok(Self::ReleaseGate),
            "blocked" => Ok(Self::Blocked),
            other => Err(TicketFlowError::InvalidStatus(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextAction {
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub action_type: Option<NextActionType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
}

impl NextAction {
    pub fn has_fields(&self) -> bool {
        self.action_type.is_some() || self.command.is_some() || self.owner.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CurrentState {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action: Option<NextAction>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CheckpointPatch {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocker: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_action_type: Option<NextActionType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_owner: Option<String>,
}

impl CheckpointPatch {
    pub fn is_empty(&self) -> bool {
        self.phase.is_none()
            && self.decision.is_none()
            && self.evidence.is_none()
            && self.blocker.is_none()
            && self.next.is_none()
            && self.note.is_none()
            && self.next_action_type.is_none()
            && self.next_command.is_none()
            && self.next_owner.is_none()
    }
}
