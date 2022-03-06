use std::path::PathBuf;
use clap::{Parser};

/// A cross-platform TUI database management tool written in Rust
#[derive(Parser)]
pub struct CliConfig {
    #[clap(short, long, parse(from_os_str), value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Database url to use
    #[clap(validator = validate_database_url)]
    pub url: Option<String>,
}

fn validate_database_url (s: &str) -> Result<(), String> {
    // TODO: regex
    if s.starts_with("mysql://") || s.starts_with("sqlite://") || s.starts_with("postgres://")  {
        return Ok(())
    }
    Err(format!("wrong database url {}", s))
}


pub fn parse() -> CliConfig {
    CliConfig::parse()
}
