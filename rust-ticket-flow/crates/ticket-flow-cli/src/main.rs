#![forbid(unsafe_code)]

mod args;
mod clawhip_cmd;
mod run;
mod support;

use std::process::ExitCode;

use clap::Parser;

use crate::args::Cli;

fn main() -> ExitCode {
    match run::run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("Error: {error}");
            ExitCode::FAILURE
        }
    }
}
