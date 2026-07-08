use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::model::{NextActionType, Ticket, TicketId, TicketStatus};
use crate::store::TicketStore;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TicketViewItem {
    pub id: TicketId,
    pub title: String,
    pub status: TicketStatus,
    pub priority: String,
    pub updated: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgentQueueItem {
    pub id: TicketId,
    pub title: String,
    pub status: TicketStatus,
    pub owner: Option<String>,
    pub command: Option<String>,
    pub next: Option<String>,
    pub updated: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardView {
    pub open: Vec<TicketViewItem>,
    pub doing: Vec<TicketViewItem>,
    pub review: Vec<TicketViewItem>,
    pub blocked: Vec<TicketViewItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewView {
    pub tickets: Vec<TicketViewItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinationView {
    pub tickets: Vec<CoordinationTicket>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoordinationTicket {
    pub id: TicketId,
    pub title: String,
    pub handoffs: usize,
    pub approvals: usize,
    pub blockers: Vec<String>,
}

pub fn agent_queue(store: &TicketStore) -> Result<Vec<AgentQueueItem>> {
    let tickets = store.list_tickets()?;
    Ok(tickets
        .into_iter()
        .filter_map(agent_queue_item)
        .collect::<Vec<_>>())
}

pub fn board(store: &TicketStore) -> Result<BoardView> {
    let mut view = BoardView {
        open: Vec::new(),
        doing: Vec::new(),
        review: Vec::new(),
        blocked: Vec::new(),
    };
    for ticket in store.list_tickets()? {
        match ticket.status {
            TicketStatus::Open => view.open.push(ticket_item(&ticket)),
            TicketStatus::Doing => view.doing.push(ticket_item(&ticket)),
            TicketStatus::Review => view.review.push(ticket_item(&ticket)),
            TicketStatus::Blocked => view.blocked.push(ticket_item(&ticket)),
            TicketStatus::Done => {}
        }
    }
    Ok(view)
}

pub fn review(store: &TicketStore) -> Result<ReviewView> {
    let tickets = store
        .list_tickets()?
        .into_iter()
        .filter(|ticket| ticket.status == TicketStatus::Review)
        .map(|ticket| ticket_item(&ticket))
        .collect();
    Ok(ReviewView { tickets })
}

pub fn coordination(store: &TicketStore) -> Result<CoordinationView> {
    let tickets = store
        .list_tickets()?
        .into_iter()
        .filter_map(|ticket| {
            ticket
                .coordination
                .as_ref()
                .map(|coordination| CoordinationTicket {
                    id: ticket.id.clone(),
                    title: ticket.title.clone(),
                    handoffs: coordination.handoffs.len(),
                    approvals: coordination.approvals.len(),
                    blockers: coordination.blockers.clone(),
                })
        })
        .collect();
    Ok(CoordinationView { tickets })
}

fn agent_queue_item(ticket: Ticket) -> Option<AgentQueueItem> {
    let current = ticket.current.as_ref()?;
    let next_action = current.next_action.as_ref()?;
    if next_action.action_type != Some(NextActionType::AgentAction) {
        return None;
    }
    Some(AgentQueueItem {
        id: ticket.id,
        title: ticket.title,
        status: ticket.status,
        owner: next_action.owner.clone(),
        command: next_action.command.clone(),
        next: current.next.clone(),
        updated: ticket.updated,
    })
}

fn ticket_item(ticket: &Ticket) -> TicketViewItem {
    TicketViewItem {
        id: ticket.id.clone(),
        title: ticket.title.clone(),
        status: ticket.status,
        priority: ticket.priority.clone(),
        updated: ticket.updated.clone(),
    }
}
