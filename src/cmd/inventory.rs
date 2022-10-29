use crate::{
    config::{ConfigOpts, ConfigOptsInventory},
    inventory,
};
use anyhow::{Context, Result};
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

        // Get token
        let token = opts
            .verify
            .as_ref()
            .map(|opts| opts.token.clone())
            .flatten()
            .context("no token was provided")?;

        let inventory = inventory::Inventory::from_file(
            opts.inventory.unwrap_or_default().path,
        )?;

        match self.action {
            InventorySubcommands::Show => println!("{:#?}", inventory),
            InventorySubcommands::Check => {
                let ip = public_ip::addr()
                    .await
                    .context("error resolving public ip")?
                    .to_string();
                println!("Public IP: {}", ip);
                if let Some(inventory) = inventory.0 {
                    for (zone_id, zone) in inventory {
                        if let Some(records) = zone.0 {
                            for record in records {
                                println!("{:?}: {:?}", zone_id, record);
                            }
                        }
                    }
                }
            }
            InventorySubcommands::Commit => todo!(),
            InventorySubcommands::Watch => todo!(),
        }

        Ok(())
    }
}
