use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(about, author, version, name = "cfddns")]
struct Args {
    #[clap(subcommand)]
    action: Subcommands,
    /// Path to the default config file [default: CFDDNS.toml]
    #[clap(short, long, env = "CFDDNS_CONFIG")]
    pub config: Option<PathBuf>,
    /// Enable verbose logging.
    #[clap(short)]
    pub v: bool,
}

impl Args {
    pub async fn run(self) -> Result<()> {
        match self.action {
            Subcommands::Show => {
                if let Some(config_path) = self.config.as_deref() {
                    println!("Config overwritten: {}", config_path.display());
                }

                // TODO parse TOML
                // TODO show config
                Ok(())
            }
        }
    }
}

#[derive(Subcommand, Debug)]
enum Subcommands {
    Show,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    args.run().await
}
