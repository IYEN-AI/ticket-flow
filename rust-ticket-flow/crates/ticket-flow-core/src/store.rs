use std::path::PathBuf;

use crate::clawhip::{ClawhipEventKind, EnvClawhipEmit, emit_ticket_event_from_env};
use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_CREATED};
use crate::model::{CreateTicket, Ticket, TicketId};
use crate::time::today_yyyymmdd;

mod checkpoint;
mod import;
mod io;
mod mutations;
mod status;

pub use import::ImportTicketStoreSummary;
pub use mutations::{AddLogInput, LinkKind, LinkTicketInput};
pub use status::StatusPatch;

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
        let now = crate::time::now_rfc3339();
        let ticket = Ticket::new(id.clone(), input, now);
        let event = EventEnvelope::new(
            id.clone(),
            TICKET_CREATED,
            serde_json::json!({ "title": ticket.title }),
        );
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
}
