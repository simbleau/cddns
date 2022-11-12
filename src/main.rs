#![feature(slice_pattern)]
#![feature(try_blocks)]
#![feature(is_some_with)]
#![feature(unwrap_infallible)]
#![feature(iter_intersperse)]
#![feature(exact_size_is_empty)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod cloudflare;
mod cmd;
mod config;
mod inventory;
mod io;

/// Cloudflare DDNS command line utility
#[derive(Parser, Debug)]
#[clap(about, author, version, name = "cddns")]
struct Args {
    #[clap(subcommand)]
    action: Subcommands,
    /// A config file to use. [default: $XDG_CONFIG_HOME/cddns/config.toml]
    #[clap(short, long, env = "CDDNS_CONFIG", value_name = "file")]
    pub config: Option<PathBuf>,
    /// TODO: Enable verbose logging.
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
