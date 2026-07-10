use std::collections::BTreeMap;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::TicketStore;
use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_LINKED, TICKET_LOG_ADDED};
use crate::model::{LinkMap, LogEntry, Ticket};
use crate::time::now_rfc3339;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinkKind {
    GithubIssues,
    Prs,
    Threads,
    CronJobs,
}

impl FromStr for LinkKind {
    type Err = TicketFlowError;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "github_issues" => Ok(Self::GithubIssues),
            "prs" => Ok(Self::Prs),
            "threads" => Ok(Self::Threads),
            "cron_jobs" => Ok(Self::CronJobs),
            other => Err(TicketFlowError::InvalidLinkKind(other.to_owned())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LinkTicketInput {
    pub kind: LinkKind,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddLogInput {
    pub note: String,
}

impl TicketStore {
    pub fn link_ticket(
        &self,
        id: &crate::model::TicketId,
        input: LinkTicketInput,
    ) -> Result<Ticket> {
        let mut ticket = self.load_active(id)?;
        let now = now_rfc3339();
        let links = links_for_kind(&mut ticket.links, input.kind);
        if !links.contains(&input.value) {
            links.push(input.value.clone());
        }
        ticket.updated = now;
        self.commit_active(
            &ticket,
            EventEnvelope::new(id.clone(), TICKET_LINKED, serde_json::to_value(input.kind)?),
        )?;
        Ok(ticket)
    }

    pub fn add_log(&self, id: &crate::model::TicketId, input: AddLogInput) -> Result<Ticket> {
        let mut ticket = self.load_active(id)?;
        let now = now_rfc3339();
        ticket.log.push(LogEntry {
            ts: now.clone(),
            action: Some("note".to_owned()),
            note: Some(input.note.clone()),
            phase: None,
            decision: None,
            evidence: None,
            blocker: None,
            next: None,
            next_action: None,
            extra: BTreeMap::<String, Value>::new(),
        });
        ticket.updated = now;
        self.commit_active(
            &ticket,
            EventEnvelope::new(id.clone(), TICKET_LOG_ADDED, json!({ "note": input.note })),
        )?;
        Ok(ticket)
    }
}

fn links_for_kind(links: &mut LinkMap, kind: LinkKind) -> &mut Vec<String> {
    match kind {
        LinkKind::GithubIssues => &mut links.github_issues,
        LinkKind::Prs => &mut links.prs,
        LinkKind::Threads => &mut links.threads,
        LinkKind::CronJobs => &mut links.cron_jobs,
    }
}
