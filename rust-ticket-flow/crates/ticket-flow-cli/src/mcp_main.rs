#![forbid(unsafe_code)]

mod mcp;

use std::process::ExitCode;

use anyhow::Result;

fn main() -> Result<ExitCode> {
    mcp::run_stdio()?;
    Ok(ExitCode::SUCCESS)
}
