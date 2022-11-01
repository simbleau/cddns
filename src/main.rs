#![feature(slice_pattern)]
#![feature(try_blocks)]
#![feature(is_some_with)]
#![feature(unwrap_infallible)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod cloudfare;
mod cmd;
mod config;
mod inventory;
mod io;

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
            Subcommands::Inventory(inner) => inner.run(self.config).await,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Config(cmd::ConfigCmd),
    Verify(cmd::VerifyCmd),
    List(cmd::ListCmd),
    Inventory(cmd::InventoryCmd),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    args.run().await
}
