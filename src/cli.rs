use std::path::PathBuf;
use clap::{Parser};

/// A cross-platform TUI database management tool written in Rust
#[derive(Parser)]
pub struct CliConfig {
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    pub config: Option<PathBuf>,
}

pub fn parse() -> CliConfig {
    CliConfig::parse()
}
