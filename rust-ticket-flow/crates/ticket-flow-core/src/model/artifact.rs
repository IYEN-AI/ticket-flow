use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use super::NextAction;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Artifact {
    #[serde(rename = "type")]
    pub artifact_type: String,
    pub value: String,
    pub ts: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

impl Artifact {
    pub fn new(artifact_type: impl Into<String>, value: impl Into<String>, ts: String) -> Self {
        Self {
            artifact_type: artifact_type.into(),
            value: value.into(),
            ts,
            extra: BTreeMap::new(),
        }
    }
}

impl<'de> Deserialize<'de> for Artifact {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ArtifactWire {
            Text(String),
            Object(ArtifactObject),
        }

        #[derive(Deserialize)]
        struct ArtifactObject {
            #[serde(default = "default_artifact_type", rename = "type")]
            artifact_type: String,
            #[serde(default)]
            value: String,
            #[serde(default)]
            ts: String,
            #[serde(flatten)]
            extra: BTreeMap<String, Value>,
        }

        fn default_artifact_type() -> String {
            "artifact".to_owned()
        }

        match ArtifactWire::deserialize(deserializer)? {
            ArtifactWire::Text(value) => Ok(Self::new("artifact", value, String::new())),
            ArtifactWire::Object(object) => Ok(Self {
                artifact_type: object.artifact_type,
                value: object.value,
                ts: object.ts,
                extra: object.extra,
            }),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LinkMap {
    #[serde(default)]
    pub github_issues: Vec<String>,
    #[serde(default)]
    pub prs: Vec<String>,
    #[serde(default)]
    pub threads: Vec<String>,
    #[serde(default)]
    pub cron_jobs: Vec<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogEntry {
    #[serde(default)]
    pub ts: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
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
    pub next_action: Option<NextAction>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}
