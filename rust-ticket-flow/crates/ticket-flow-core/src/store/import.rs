use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::json;

use super::TicketStore;
use super::io::{archive_month_from_timestamp, is_archive_month};
use crate::error::{Result, TicketFlowError};
use crate::event::{EventEnvelope, TICKET_IMPORTED};
use crate::model::{Ticket, TicketStatus};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportTicketStoreSummary {
    pub imported: usize,
    pub active: usize,
    pub archived: usize,
}

struct SourceTicket {
    ticket: Ticket,
    source_archive_month: Option<String>,
}

impl TicketStore {
    pub fn import_store(&self, source_root: impl AsRef<Path>) -> Result<ImportTicketStoreSummary> {
        let source_root = source_root.as_ref();
        if !source_root.is_dir() {
            return Err(invalid_import_source(source_root));
        }
        if fs::read_dir(source_root).is_err() {
            return Err(invalid_import_source(source_root));
        }
        let source_tickets = match read_source_tickets(source_root) {
            Ok(tickets) => tickets,
            Err(TicketFlowError::Io(_)) => return Err(invalid_import_source(source_root)),
            Err(error) => return Err(error),
        };
        reject_duplicate_sources(&source_tickets)?;
        self.ensure_store()?;
        self.reject_destination_collisions(&source_tickets)?;

        let mut summary = ImportTicketStoreSummary {
            imported: source_tickets.len(),
            active: 0,
            archived: 0,
        };
        let mut index = self.read_index()?;
        for source_ticket in source_tickets {
            let id = source_ticket.ticket.id.clone();
            self.append_event(&EventEnvelope::new(
                id.clone(),
                TICKET_IMPORTED,
                json!({ "source_root": source_root.display().to_string() }),
            ))?;
            if source_ticket.ticket.status == TicketStatus::Done {
                let month = archive_month(&source_ticket);
                self.write_ticket_archived_month(&source_ticket.ticket, &month)?;
                index.tickets.remove(&id);
                summary.archived += 1;
            } else {
                self.write_ticket_active(&source_ticket.ticket)?;
                index.tickets.insert(id, source_ticket.ticket.summary());
                summary.active += 1;
            }
        }
        self.write_index(&index)?;
        Ok(summary)
    }

    fn reject_destination_collisions(&self, source_tickets: &[SourceTicket]) -> Result<()> {
        let mut destination_ids = self.ticket_ids_in_dir(&self.paths.active)?;
        destination_ids.extend(self.archive_ticket_ids()?);
        for source_ticket in source_tickets {
            if destination_ids.contains(&source_ticket.ticket.id) {
                return Err(TicketFlowError::DuplicateDestinationTicket(
                    source_ticket.ticket.id.to_string(),
                ));
            }
        }
        Ok(())
    }
}

fn invalid_import_source(source_root: &Path) -> TicketFlowError {
    TicketFlowError::InvalidImportSource(source_root.display().to_string())
}

fn read_source_tickets(source_root: &Path) -> Result<Vec<SourceTicket>> {
    let mut tickets = read_source_dir(&source_root.join("active"), None)?;
    tickets.extend(read_source_archive(&source_root.join("archive"))?);
    Ok(tickets)
}

fn read_source_archive(archive_root: &Path) -> Result<Vec<SourceTicket>> {
    let mut tickets = read_source_dir(archive_root, None)?;
    if !archive_root.exists() {
        return Ok(tickets);
    }
    for entry in fs::read_dir(archive_root)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(month) = name.to_str() else {
            continue;
        };
        if !is_archive_month(month) {
            continue;
        }
        tickets.extend(read_source_dir(&entry.path(), Some(month.to_owned()))?);
    }
    Ok(tickets)
}

fn read_source_dir(dir: &Path, source_archive_month: Option<String>) -> Result<Vec<SourceTicket>> {
    let mut tickets = Vec::new();
    for path in ticket_files(dir)? {
        let data = fs::read(path)?;
        tickets.push(SourceTicket {
            ticket: serde_json::from_slice(&data)?,
            source_archive_month: source_archive_month.clone(),
        });
    }
    Ok(tickets)
}

fn ticket_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut paths = Vec::new();
    if !dir.exists() {
        return Ok(paths);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

fn reject_duplicate_sources(source_tickets: &[SourceTicket]) -> Result<()> {
    let mut seen = BTreeSet::new();
    for source_ticket in source_tickets {
        if !seen.insert(source_ticket.ticket.id.clone()) {
            return Err(TicketFlowError::DuplicateSourceTicket(
                source_ticket.ticket.id.to_string(),
            ));
        }
    }
    Ok(())
}

fn archive_month(source_ticket: &SourceTicket) -> String {
    source_ticket
        .ticket
        .closed
        .as_deref()
        .and_then(archive_month_from_timestamp)
        .or_else(|| source_ticket.source_archive_month.clone())
        .or_else(|| archive_month_from_timestamp(&source_ticket.ticket.updated))
        .unwrap_or_else(|| "unknown".to_owned())
}
