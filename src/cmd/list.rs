use std::path::PathBuf;

use crate::config::{ConfigOpts, ConfigOptsList};
use anyhow::Result;
use clap::{Args, Subcommand};

/// List Cloudfare resources
#[derive(Debug, Args)]
#[clap(name = "list")]
pub struct List {
    #[clap(subcommand)]
    action: Option<ListSubcommands>,
    #[clap(flatten)]
    pub cfg: ConfigOptsList,
}

impl List {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            list: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);
        println!("{:#?}", opts);

        match self.action {
            Some(filter) => match filter {
                ListSubcommands::Zones => {
                    todo!("Print cloudfare zones")
                }
                ListSubcommands::Records => {
                    todo!("Print cloudfare records")
                }
            },
            None => {
                todo!("Print cloudfare zones and records")
            }
        }
    }
}

#[derive(Clone, Debug, Subcommand)]
enum ListSubcommands {
    /// Show zones (domains, subdomains, and identities)
    Zones,
    /// Show authoritative DNS records
    Records,
}
