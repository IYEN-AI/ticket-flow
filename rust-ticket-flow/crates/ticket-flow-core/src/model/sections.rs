use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::Artifact;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    #[default]
    None,
    Partial,
    Sufficient,
    Stale,
    Disputed,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SharedSection {
    #[serde(default)]
    pub constraints: Vec<String>,
    #[serde(default)]
    pub assumptions: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CoordinationSection {
    #[serde(default)]
    pub owners: Vec<String>,
    #[serde(default)]
    pub handoffs: Vec<HandoffRecord>,
    #[serde(default)]
    pub approvals: Vec<ApprovalRecord>,
    #[serde(default)]
    pub blockers: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffRecord {
    pub id: String,
    pub from: String,
    pub to: String,
    pub reason: String,
    pub status: String,
    pub requested_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acknowledged_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalRecord {
    pub id: String,
    pub owner: String,
    pub question: String,
    pub status: String,
    pub requested_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub outcome: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub responded_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct EvidenceSection {
    #[serde(default)]
    pub artifacts: Vec<Artifact>,
    #[serde(default)]
    pub evidence_status: EvidenceStatus,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ExecutionSection {
    #[serde(default)]
    pub commands: Vec<String>,
    #[serde(default)]
    pub checkpoints: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct MemorySection {
    #[serde(default)]
    pub notes: Vec<String>,
    #[serde(default)]
    pub decisions: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LocalSection {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub working_dir: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub machine: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
