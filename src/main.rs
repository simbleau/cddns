#![feature(slice_pattern)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod cloudfare;
mod cmd;
mod config;
mod inventory;

/// Cloudfare DDNS command line utility
#[derive(Parser, Debug)]
#[clap(about, author, version, name = "cfddns")]
struct Args {
    #[clap(subcommand)]
    action: Subcommands,
    /// Path to the default config file [default: CFDDNS.toml]
    #[clap(short, long, env = "CFDDNS_CONFIG", value_name = "file")]
    pub config: Option<PathBuf>,
    /// Enable verbose logging.
    #[clap(short)]
    pub v: bool,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        match self.action {
            Subcommands::Config(inner) => inner.run(self.config).await,
            Subcommands::Verify(inner) => inner.run(self.config).await,
            Subcommands::List(inner) => inner.run(self.config).await,
            Subcommands::Build(inner) => inner.run().await,
            Subcommands::Check => todo!(),
            Subcommands::Run => todo!(),
            Subcommands::Watch => todo!(),
        }
    }
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Config(cmd::config::Config),
    Verify(cmd::verify::Verify),
    List(cmd::list::List),
    Build(cmd::build::Build),
    Check,
    Run,
    Watch,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    args.run().await
}
