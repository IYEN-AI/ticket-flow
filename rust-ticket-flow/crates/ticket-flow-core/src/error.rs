use crate::model::TicketStatus;

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum TicketFlowError {
    #[error("approval outcome not allowed")]
    ApprovalOutcomeNotAllowed,
    #[error("blocked gate failed: needed_from or next_owner required")]
    BlockedGateFailed,
    #[error("duplicate destination ticket {0}")]
    DuplicateDestinationTicket(String),
    #[error("duplicate source ticket {0}")]
    DuplicateSourceTicket(String),
    #[error("checkpoint requires at least one field")]
    EmptyCheckpoint,
    #[error("evidence value must not be empty")]
    EmptyEvidence,
    #[error("home directory is not available")]
    HomeDirectoryUnavailable,
    #[error("id sequence exhausted for date {0}")]
    IdSequenceExhausted(String),
    #[error("invalid source {0}")]
    InvalidSource(String),
    #[error("invalid status {0}")]
    InvalidStatus(String),
    #[error("invalid link kind {0}")]
    InvalidLinkKind(String),
    #[error("invalid clawhip event kind {0}")]
    InvalidClawhipEventKind(String),
    #[error("invalid ticket id {0}")]
    InvalidTicketId(String),
    #[error("invalid transition {from} -> {to}")]
    InvalidTransition {
        from: TicketStatus,
        to: TicketStatus,
    },
    #[error("approval {0} not found")]
    ApprovalNotFound(String),
    #[error("handoff {0} not found")]
    HandoffNotFound(String),
    #[error("review requires --artifact")]
    ReviewArtifactRequired,
    #[error("ticket {0} not found")]
    TicketNotFound(String),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Http(#[from] reqwest::Error),
}

pub type Result<T> = std::result::Result<T, TicketFlowError>;
