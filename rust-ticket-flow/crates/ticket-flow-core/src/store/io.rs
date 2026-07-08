use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::Serialize;
use uuid::Uuid;

use super::TicketStore;
use crate::error::{Result, TicketFlowError};
use crate::event::EventEnvelope;
use crate::model::{Ticket, TicketId, TicketIndex};

impl TicketStore {
    pub fn ensure_store(&self) -> Result<()> {
        fs::create_dir_all(&self.paths.active)?;
        fs::create_dir_all(&self.paths.archive)?;
        fs::create_dir_all(&self.paths.events)?;
        if !self.paths.index.exists() {
            self.write_json_atomic(&self.paths.index, &TicketIndex::default())?;
        }
        Ok(())
    }

    pub(crate) fn load_active(&self, id: &TicketId) -> Result<Ticket> {
        self.ensure_store()?;
        let path = self.ticket_path(&self.paths.active, id);
        if path.exists() {
            self.read_ticket(&path)
        } else {
            Err(TicketFlowError::TicketNotFound(id.to_string()))
        }
    }

    pub(crate) fn commit_active(&self, ticket: &Ticket, event: EventEnvelope) -> Result<()> {
        self.append_event(&event)?;
        self.write_ticket_active(ticket)?;
        let mut index = self.read_index()?;
        index.tickets.insert(ticket.id.clone(), ticket.summary());
        self.write_index(&index)
    }

    pub(crate) fn append_event(&self, event: &EventEnvelope) -> Result<()> {
        self.ensure_store()?;
        let path = self
            .paths
            .events
            .join(format!("{}.ndjson", event.ticket_id.as_str()));
        let mut file = OpenOptions::new().create(true).append(true).open(path)?;
        serde_json::to_writer(&mut file, event)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    pub(crate) fn archive_ticket(&self, ticket: &Ticket, event: EventEnvelope) -> Result<()> {
        self.append_event(&event)?;
        let month = archive_month_for_ticket(ticket);
        self.write_ticket_archived_month(ticket, &month)?;
        let active = self.ticket_path(&self.paths.active, &ticket.id);
        if active.exists() {
            fs::remove_file(active)?;
        }
        let mut index = self.read_index()?;
        index.tickets.remove(&ticket.id);
        self.write_index(&index)
    }

    pub(crate) fn read_index(&self) -> Result<TicketIndex> {
        self.ensure_store()?;
        if !self.paths.index.exists() {
            return Ok(TicketIndex::default());
        }
        let data = fs::read(&self.paths.index)?;
        Ok(serde_json::from_slice(&data)?)
    }

    pub(crate) fn write_index(&self, index: &TicketIndex) -> Result<()> {
        self.write_json_atomic(&self.paths.index, index)
    }

    pub(crate) fn read_ticket(&self, path: &Path) -> Result<Ticket> {
        let data = fs::read(path)?;
        Ok(serde_json::from_slice(&data)?)
    }

    pub(crate) fn write_ticket_active(&self, ticket: &Ticket) -> Result<()> {
        self.write_json_atomic(&self.ticket_path(&self.paths.active, &ticket.id), ticket)
    }

    pub(crate) fn write_ticket_archived_month(&self, ticket: &Ticket, month: &str) -> Result<()> {
        let path = self
            .paths
            .archive
            .join(month)
            .join(format!("{}.json", ticket.id));
        self.write_json_atomic(&path, ticket)
    }

    pub(crate) fn ticket_path(&self, dir: &Path, id: &TicketId) -> PathBuf {
        dir.join(format!("{}.json", id.as_str()))
    }

    pub(crate) fn next_ticket_id(&self, day: &str) -> Result<TicketId> {
        let mut max_suffix = self
            .read_index()?
            .last_id
            .as_ref()
            .and_then(|id| suffix_for_day(id, day))
            .unwrap_or(0);
        for id in self.ticket_ids_in_dir(&self.paths.active)? {
            max_suffix = max_suffix.max(suffix_for_day(&id, day).unwrap_or(0));
        }
        for id in self.archive_ticket_ids()? {
            max_suffix = max_suffix.max(suffix_for_day(&id, day).unwrap_or(0));
        }
        let next = max_suffix
            .checked_add(1)
            .ok_or_else(|| TicketFlowError::IdSequenceExhausted(day.to_owned()))?;
        TicketId::from_parts(day, next)
    }

    pub(crate) fn ticket_ids_in_dir(&self, dir: &Path) -> Result<BTreeSet<TicketId>> {
        let mut ids = BTreeSet::new();
        if !dir.exists() {
            return Ok(ids);
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let file_name = entry.file_name();
            let Some(name) = file_name.to_str() else {
                continue;
            };
            let Some(stem) = name.strip_suffix(".json") else {
                continue;
            };
            if let Ok(id) = TicketId::parse(stem) {
                ids.insert(id);
            }
        }
        Ok(ids)
    }

    pub(crate) fn archived_ticket_path(&self, id: &TicketId) -> Result<Option<PathBuf>> {
        let legacy = self.ticket_path(&self.paths.archive, id);
        if legacy.exists() {
            return Ok(Some(legacy));
        }
        for month in self.archive_month_dirs()? {
            let path = self.paths.archive.join(month).join(format!("{id}.json"));
            if path.exists() {
                return Ok(Some(path));
            }
        }
        Ok(None)
    }

    pub(crate) fn archive_ticket_ids(&self) -> Result<BTreeSet<TicketId>> {
        let mut ids = self.ticket_ids_in_dir(&self.paths.archive)?;
        for month in self.archive_month_dirs()? {
            ids.extend(self.ticket_ids_in_dir(&self.paths.archive.join(month))?);
        }
        Ok(ids)
    }

    pub(crate) fn archive_month_dirs(&self) -> Result<BTreeSet<String>> {
        let mut months = BTreeSet::new();
        if !self.paths.archive.exists() {
            return Ok(months);
        }
        for entry in fs::read_dir(&self.paths.archive)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let name = entry.file_name();
            let Some(month) = name.to_str() else {
                continue;
            };
            if is_archive_month(month) {
                months.insert(month.to_owned());
            }
        }
        Ok(months)
    }

    fn write_json_atomic<T>(&self, path: &Path, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let temp = path.with_extension(format!("tmp-{}.json", Uuid::now_v7()));
        let data = serde_json::to_vec_pretty(value)?;
        fs::write(&temp, data)?;
        fs::rename(temp, path)?;
        Ok(())
    }
}

fn suffix_for_day(id: &TicketId, day: &str) -> Option<u16> {
    let prefix = format!("T-{day}-");
    id.as_str()
        .strip_prefix(&prefix)
        .and_then(|suffix| suffix.parse::<u16>().ok())
}

fn archive_month_for_ticket(ticket: &Ticket) -> String {
    ticket
        .closed
        .as_deref()
        .and_then(archive_month_from_timestamp)
        .or_else(|| archive_month_from_timestamp(&ticket.updated))
        .unwrap_or_else(|| "unknown".to_owned())
}

pub(crate) fn archive_month_from_timestamp(value: &str) -> Option<String> {
    let month = value.chars().take(7).collect::<String>();
    if is_archive_month(&month) {
        Some(month)
    } else {
        None
    }
}

pub(crate) fn is_archive_month(value: &str) -> bool {
    let mut parts = value.split('-');
    let Some(year) = parts.next() else {
        return false;
    };
    let Some(month) = parts.next() else {
        return false;
    };
    parts.next().is_none()
        && year.len() == 4
        && year.chars().all(|c| c.is_ascii_digit())
        && month.len() == 2
        && month.chars().all(|c| c.is_ascii_digit())
}
