use crate::config::{ConfigOpts, ConfigOptsInventory};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

/// Manage inventory of watched records
#[derive(Debug, Args)]
#[clap(name = "inventory")]
pub struct Inventory {
    #[clap(subcommand)]
    action: InventorySubcommands,
    #[clap(flatten)]
    pub cfg: ConfigOptsInventory,
}

#[derive(Clone, Debug, Subcommand)]
enum InventorySubcommands {
    /// Print your inventory
    Show,
    /// Print erroneous DNS records
    Check,
    /// Fix erroneous DNS records once
    Commit,
    /// Fix erroneous DNS records on a loop
    Watch,
}

impl Inventory {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            inventory: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);

        Ok(())
    }
}
