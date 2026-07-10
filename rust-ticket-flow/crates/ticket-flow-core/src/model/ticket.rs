use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    Artifact, CoordinationSection, CurrentState, EvidenceSection, ExecutionSection, LinkMap,
    LocalSection, LogEntry, MemorySection, SharedSection, TicketId, TicketStatus,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ticket {
    pub id: TicketId,
    pub title: String,
    #[serde(default)]
    pub status: TicketStatus,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default = "default_ticket_type", rename = "type")]
    pub ticket_type: String,
    #[serde(default)]
    pub source: Option<Value>,
    #[serde(default)]
    pub goal: String,
    #[serde(default)]
    pub acceptance: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub links: LinkMap,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub children: Vec<String>,
    #[serde(default = "default_assignee")]
    pub assignee: String,
    pub created: String,
    pub updated: String,
    #[serde(default)]
    pub closed: Option<String>,
    #[serde(default)]
    pub log: Vec<LogEntry>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current: Option<CurrentState>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shared: Option<SharedSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordination: Option<CoordinationSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence: Option<EvidenceSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionSection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub memory: Option<MemorySection>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local: Option<LocalSection>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Ticket {
    pub fn new(id: TicketId, input: CreateTicket, now: String) -> Self {
        Self {
            id,
            title: input.title,
            status: TicketStatus::Open,
            priority: input.priority,
            ticket_type: input.ticket_type,
            source: input.source,
            goal: input.goal,
            acceptance: input.acceptance,
            tags: input.tags,
            artifacts: Vec::new(),
            links: LinkMap::default(),
            parent: input.parent,
            children: Vec::new(),
            assignee: input.assignee,
            created: now.clone(),
            updated: now,
            closed: None,
            log: Vec::new(),
            current: None,
            shared: None,
            coordination: None,
            evidence: None,
            execution: None,
            memory: None,
            local: None,
            extra: BTreeMap::new(),
        }
    }

    pub fn summary(&self) -> TicketSummary {
        TicketSummary {
            title: self.title.clone(),
            status: self.status,
            priority: self.priority.clone(),
            ticket_type: self.ticket_type.clone(),
            updated: self.updated.clone(),
            extra: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTicket {
    pub title: String,
    pub ticket_type: String,
    pub priority: String,
    pub goal: String,
    pub parent: Option<String>,
    pub assignee: String,
    pub acceptance: Vec<String>,
    pub tags: Vec<String>,
    pub source: Option<Value>,
}

impl CreateTicket {
    pub fn new(title: String) -> Self {
        Self {
            title,
            ticket_type: default_ticket_type(),
            priority: default_priority(),
            goal: String::new(),
            parent: None,
            assignee: default_assignee(),
            acceptance: Vec::new(),
            tags: Vec::new(),
            source: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketSummary {
    pub title: String,
    pub status: TicketStatus,
    pub priority: String,
    #[serde(rename = "type")]
    pub ticket_type: String,
    pub updated: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TicketIndex {
    #[serde(default = "default_index_version")]
    pub version: u32,
    #[serde(default, rename = "lastId")]
    pub last_id: Option<TicketId>,
    #[serde(default)]
    pub tickets: BTreeMap<TicketId, TicketSummary>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Default for TicketIndex {
    fn default() -> Self {
        Self {
            version: default_index_version(),
            last_id: None,
            tickets: BTreeMap::new(),
            extra: BTreeMap::new(),
        }
    }
}

fn default_priority() -> String {
    "medium".to_owned()
}

fn default_ticket_type() -> String {
    "chore".to_owned()
}

fn default_assignee() -> String {
    "iyen".to_owned()
}

fn default_index_version() -> u32 {
    1
}
