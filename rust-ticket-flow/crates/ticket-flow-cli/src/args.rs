use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "ticket-flow", version, about = "Agent-native ticket-flow CLI")]
pub struct Cli {
    #[arg(long, env = "TICKET_FLOW_HOME", global = true)]
    pub home: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Pretty, global = true)]
    pub format: OutputFormat,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    Pretty,
    Json,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    Create(CreateArgs),
    Show(IdArgs),
    List(ListArgs),
    Status(StatusArgs),
    Checkpoint(CheckpointArgs),
    Import(ImportArgs),
    ReadyCheck(IdArgs),
    Evidence {
        #[command(subcommand)]
        command: EvidenceCommand,
    },
    Handoff {
        #[command(subcommand)]
        command: HandoffCommand,
    },
    Approval {
        #[command(subcommand)]
        command: ApprovalCommand,
    },
    Clawhip {
        #[command(subcommand)]
        command: ClawhipCommand,
    },
    View {
        #[command(subcommand)]
        command: ViewCommand,
    },
    ContextPack(ContextPackArgs),
}

#[derive(Debug, Args)]
pub struct CreateArgs {
    #[arg(long)]
    pub title: String,
    #[arg(long = "type", default_value = "chore")]
    pub ticket_type: String,
    #[arg(long, default_value = "medium")]
    pub priority: String,
    #[arg(long, default_value = "")]
    pub goal: String,
    #[arg(long)]
    pub parent: Option<String>,
    #[arg(long, default_value = "iyen")]
    pub assignee: String,
    #[arg(long)]
    pub acceptance: Vec<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long)]
    pub source: Option<String>,
}

#[derive(Debug, Args)]
pub struct IdArgs {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct ListArgs {
    #[arg(long)]
    pub status: Option<String>,
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    pub id: String,
    pub status: String,
    #[arg(long)]
    pub artifact: Option<String>,
    #[arg(long)]
    pub evidence: Option<String>,
    #[arg(long)]
    pub note: Option<String>,
}

#[derive(Debug, Args)]
pub struct CheckpointArgs {
    pub id: String,
    #[arg(long)]
    pub phase: Option<String>,
    #[arg(long)]
    pub decision: Option<String>,
    #[arg(long)]
    pub evidence: Option<String>,
    #[arg(long)]
    pub blocker: Option<String>,
    #[arg(long)]
    pub next: Option<String>,
    #[arg(long)]
    pub note: Option<String>,
    #[arg(long = "next-type")]
    pub next_type: Option<String>,
    #[arg(long = "next-command")]
    pub next_command: Option<String>,
    #[arg(long = "next-owner")]
    pub next_owner: Option<String>,
}

#[derive(Debug, Args)]
pub struct ImportArgs {
    pub source: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum EvidenceCommand {
    Attach(EvidenceAttachArgs),
}

#[derive(Debug, Args)]
pub struct EvidenceAttachArgs {
    pub id: String,
    #[arg(long = "type", default_value = "artifact")]
    pub artifact_type: String,
    #[arg(long)]
    pub value: String,
}

#[derive(Debug, Subcommand)]
pub enum HandoffCommand {
    Request(HandoffRequestArgs),
    Ack(HandoffAckArgs),
}

#[derive(Debug, Args)]
pub struct HandoffRequestArgs {
    pub id: String,
    #[arg(long)]
    pub from: String,
    #[arg(long)]
    pub to: String,
    #[arg(long)]
    pub reason: String,
}

#[derive(Debug, Args)]
pub struct HandoffAckArgs {
    pub id: String,
    #[arg(long = "handoff-id")]
    pub handoff_id: String,
}

#[derive(Debug, Subcommand)]
pub enum ApprovalCommand {
    Request(ApprovalRequestArgs),
    Respond(ApprovalRespondArgs),
}

#[derive(Debug, Args)]
pub struct ApprovalRequestArgs {
    pub id: String,
    #[arg(long)]
    pub owner: String,
    #[arg(long)]
    pub question: String,
}

#[derive(Debug, Args)]
pub struct ApprovalRespondArgs {
    pub id: String,
    #[arg(long = "approval-id")]
    pub approval_id: String,
    #[arg(long)]
    pub decision: String,
}

#[derive(Debug, Subcommand)]
pub enum ClawhipCommand {
    Event(ClawhipEventArgs),
}

#[derive(Debug, Args)]
pub struct ClawhipEventArgs {
    pub id: String,
    #[arg(long)]
    pub kind: String,
    #[arg(long)]
    pub print: bool,
    #[arg(long)]
    pub send: bool,
    #[arg(long, default_value = "http://127.0.0.1:25294")]
    pub url: String,
    #[arg(long = "timeout-ms", default_value_t = 1_000)]
    pub timeout_ms: u64,
}

#[derive(Debug, Subcommand)]
pub enum ViewCommand {
    AgentQueue,
    Board,
    Review,
    Coordination,
}

#[derive(Debug, Args)]
pub struct ContextPackArgs {
    pub id: String,
    #[arg(long, default_value = "agent_execution")]
    pub audience: String,
}
