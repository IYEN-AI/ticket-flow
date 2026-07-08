use std::path::PathBuf;
use std::process::ExitCode;
use std::str::FromStr;

use anyhow::Result;
use serde::Serialize;
use serde_json::json;
use ticket_flow_core::{
    ApprovalDecision, ContextAudience, NextActionType, StorePaths, TicketStatus,
};

use crate::args::OutputFormat;

pub(crate) fn paths(home: Option<PathBuf>) -> Result<StorePaths> {
    match home {
        Some(root) => Ok(StorePaths::new(root)),
        None => Ok(StorePaths::from_env_or_default()?),
    }
}

pub(crate) fn print_subject<T: Serialize>(
    format: OutputFormat,
    value: &T,
    pretty: &str,
) -> Result<ExitCode> {
    print_json_or_pretty(format, value, pretty)?;
    Ok(ExitCode::SUCCESS)
}

pub(crate) fn print_json_or_pretty<T: Serialize>(
    format: OutputFormat,
    value: &T,
    pretty: &str,
) -> Result<()> {
    match format {
        OutputFormat::Pretty => {
            println!("{pretty}");
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(value)?);
            Ok(())
        }
    }
}

pub(crate) fn parse_optional_next_action(value: Option<String>) -> Result<Option<NextActionType>> {
    value
        .map(|text| NextActionType::from_str(&text))
        .transpose()
        .map_err(Into::into)
}

pub(crate) fn parse_status(value: &str) -> Result<TicketStatus> {
    TicketStatus::from_str(value).map_err(Into::into)
}

pub(crate) fn parse_approval_decision(value: &str) -> Result<ApprovalDecision> {
    match value {
        "approved" => Ok(ApprovalDecision::Approved),
        "rejected" => Ok(ApprovalDecision::Rejected),
        other => anyhow::bail!("approval decision must be approved or rejected, got {other}"),
    }
}

pub(crate) fn parse_audience(value: &str) -> Result<ContextAudience> {
    match value {
        "agent_execution" => Ok(ContextAudience::AgentExecution),
        "owner_review" => Ok(ContextAudience::OwnerReview),
        "release_review" => Ok(ContextAudience::ReleaseReview),
        other => anyhow::bail!("unknown context audience {other}"),
    }
}

pub(crate) fn parse_source(value: Option<String>) -> Result<Option<serde_json::Value>> {
    let Some(source) = value else {
        return Ok(None);
    };
    let Some((kind, reference)) = source.split_once(':') else {
        anyhow::bail!("invalid source {source}");
    };
    if kind.trim().is_empty() || reference.trim().is_empty() {
        anyhow::bail!("invalid source {source}");
    }
    Ok(Some(json!({ "type": kind, "ref": reference })))
}
