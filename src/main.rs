use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
mod cmd;
mod config;

/// Cloudfare DDNS CLI arguments
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
            Subcommands::Check(inner) => inner.run(self.config).await,
        }
    }
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Config(cmd::config::Config),
    Check(cmd::check::Check),
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    args.run().await
}
