use std::process::ExitCode;

use clap::Parser;
use nginxfmt::cli::Cli;

fn main() -> ExitCode {
    Cli::parse().run()
}
