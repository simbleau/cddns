use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::config::ConfigOpts;

/// Configuration controls
#[derive(Debug, Args)]
#[clap(name = "config")]
pub struct Config {
    #[clap(subcommand)]
    action: ConfigSubcommands,
}

impl Config {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        match self.action {
            ConfigSubcommands::Show => {
                let cfg = ConfigOpts::full(config)?;
                println!("{:#?}", cfg);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
enum ConfigSubcommands {
    /// Show the current config pre-CLI.
    Show,
}
