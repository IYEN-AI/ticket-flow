use std::path::PathBuf;

use serde_json::json;

use crate::clawhip::{ClawhipEventKind, EnvClawhipEmit, emit_ticket_event_from_env};
use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_CREATED, TICKET_STATUS_CHANGED};
use crate::model::{Artifact, CreateTicket, Ticket, TicketId, TicketStatus};
use crate::time::{now_rfc3339, today_yyyymmdd};

mod checkpoint;
mod import;
mod io;
mod mutations;

pub use import::ImportTicketStoreSummary;
pub use mutations::{AddLogInput, LinkKind, LinkTicketInput};

#[derive(Debug, Clone)]
pub struct StorePaths {
    pub root: PathBuf,
    pub active: PathBuf,
    pub archive: PathBuf,
    pub events: PathBuf,
    pub index: PathBuf,
}

impl StorePaths {
    pub fn new(root: PathBuf) -> Self {
        Self {
            active: root.join("active"),
            archive: root.join("archive"),
            events: root.join("events"),
            index: root.join("index.json"),
            root,
        }
    }

    pub fn from_env_or_default() -> Result<Self> {
        match std::env::var_os("TICKET_FLOW_HOME") {
            Some(value) => Ok(Self::new(PathBuf::from(value))),
            None => {
                let home = dirs::home_dir().ok_or(TicketFlowError::HomeDirectoryUnavailable)?;
                Ok(Self::new(home.join(".ticket-flow").join("tickets")))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TicketStore {
    paths: StorePaths,
}

impl TicketStore {
    pub fn new(paths: StorePaths) -> Self {
        Self { paths }
    }

    pub fn paths(&self) -> &StorePaths {
        &self.paths
    }

    pub fn create_ticket(&self, input: CreateTicket) -> Result<Ticket> {
        self.ensure_store()?;
        let id = self.next_ticket_id(&today_yyyymmdd())?;
        let now = now_rfc3339();
        let ticket = Ticket::new(id.clone(), input, now);
        let event =
            EventEnvelope::new(id.clone(), TICKET_CREATED, json!({ "title": ticket.title }));
        self.append_event(&event)?;
        self.write_ticket_active(&ticket)?;
        let mut index = self.read_index()?;
        index.last_id = Some(id);
        index.tickets.insert(ticket.id.clone(), ticket.summary());
        self.write_index(&index)?;
        emit_ticket_event_from_env(EnvClawhipEmit {
            kind: ClawhipEventKind::Created,
            ticket: &ticket,
            from_status: None,
            to_status: None,
        });
        Ok(ticket)
    }

    pub fn get_ticket(&self, id: &TicketId) -> Result<Ticket> {
        self.ensure_store()?;
        let active = self.ticket_path(&self.paths.active, id);
        if active.exists() {
            return self.read_ticket(&active);
        }
        if let Some(archived) = self.archived_ticket_path(id)? {
            return self.read_ticket(&archived);
        }
        Err(TicketFlowError::TicketNotFound(id.to_string()))
    }

    pub fn list_tickets(&self) -> Result<Vec<Ticket>> {
        self.ensure_store()?;
        let mut tickets = Vec::new();
        for id in self.ticket_ids_in_dir(&self.paths.active)? {
            tickets.push(self.get_ticket(&id)?);
        }
        Ok(tickets)
    }

    pub fn update_status(&self, id: &TicketId, patch: StatusPatch) -> Result<Ticket> {
        let mut ticket = self.load_active(id)?;
        if patch.status == TicketStatus::Review && patch.artifact.is_none() {
            return Err(TicketFlowError::ReviewArtifactRequired);
        }
        let from = ticket.status;
        ticket.status = patch.status;
        ticket.updated = now_rfc3339();
        if patch.status == TicketStatus::Done {
            ticket.closed = Some(ticket.updated.clone());
        }
        if let Some(value) = patch.artifact {
            ticket
                .artifacts
                .push(Artifact::new("artifact", value, ticket.updated.clone()));
        }
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatusPatch {
    pub status: TicketStatus,
    pub artifact: Option<String>,
    pub evidence: Option<String>,
    pub note: Option<String>,
}
