#![forbid(unsafe_code)]

pub mod approval;
pub mod clawhip;
pub mod context_pack;
pub mod error;
pub mod event;
pub mod evidence;
pub mod gates;
pub mod handoff;
pub mod model;
pub mod store;
pub mod time;
pub mod views;

pub use approval::{ApprovalDecision, ApprovalRequest, request_approval, respond_approval};
pub use context_pack::{ContextAudience, ContextPack, build_context_pack};
pub use error::{Result, TicketFlowError};
pub use evidence::{EvidenceInput, attach_evidence};
pub use gates::{GateResult, blocked_gate, done_gate, ready_gate, review_gate};
pub use handoff::{HandoffAck, HandoffRequest, ack_handoff, request_handoff};
pub use model::{
    Artifact, CheckpointPatch, CreateTicket, EvidenceStatus, NextAction, NextActionType, Ticket,
    TicketId, TicketIndex, TicketStatus,
};
pub use store::{
    AddLogInput, ImportTicketStoreSummary, LinkKind, LinkTicketInput, StatusPatch, StorePaths,
    TicketStore,
};
pub use views::{
    AgentQueueItem, BoardView, CoordinationView, ReviewView, agent_queue, board, coordination,
    review,
};
