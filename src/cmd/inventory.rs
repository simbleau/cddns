use crate::{
    cloudfare::{self, models::Record},
    config::{ConfigOpts, ConfigOptsInventory},
    inventory::{
        Inventory, InventoryRecord, InventoryZone, DEFAULT_INVENTORY_PATH,
    },
    io::{self, Scanner},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{collections::HashMap, path::PathBuf};

/// Manage inventory of watched records
#[derive(Debug, Args)]
#[clap(name = "inventory")]
pub struct InventoryCmd {
    #[clap(subcommand)]
    action: InventorySubcommands,
    #[clap(flatten)]
    pub cfg: ConfigOptsInventory,
}

#[derive(Clone, Debug, Subcommand)]
enum InventorySubcommands {
    /// Build an inventory file.
    Build,
    /// Print your inventory.
    Show,
    /// Print erroneous DNS records.
    Check,
    /// Fix erroneous DNS records once.
    Commit,
    /// Fix erroneous DNS records on a loop.
    Watch,
}

impl InventoryCmd {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            inventory: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);

        match self.action {
            InventorySubcommands::Build => {
                let runtime = tokio::runtime::Handle::current();
                let mut scanner = Scanner::new(runtime);

                // Build
                let inventory = build(&mut scanner, &opts).await?;

                // Save
                let path = scanner
                    .prompt_path_or(
                        format!(
                            "Save location [default: {}]",
                            DEFAULT_INVENTORY_PATH
                        ),
                        DEFAULT_INVENTORY_PATH.into(),
                    )
                    .await?;
                if path.exists() {
                    io::fs::remove_interactive(&path, &mut scanner).await?;
                }
                io::fs::save_yaml(&inventory, path).await?;
                println!("Saved");
            }
            InventorySubcommands::Show => {
                let inventory = Inventory::from_file(
                    opts.inventory.unwrap_or_default().path,
                )?;
                println!("{:#?}", inventory)
            }
            InventorySubcommands::Check => {
                let ip = public_ip::addr()
                    .await
                    .context("error resolving public ip")?
                    .to_string();
                let inventory = Inventory::from_file(
                    opts.inventory.unwrap_or_default().path,
                )?;
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

async fn build(scanner: &mut Scanner, opts: &ConfigOpts) -> Result<Inventory> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .map(|opts| opts.token.clone())
        .flatten()
        .context("no token was provided")?;

    let mut zones = cloudfare::endpoints::zones(&token).await?;
    crate::cmd::list::filter_zones(&mut zones, opts)?;
    let mut records = cloudfare::endpoints::records(&zones, &token).await?;
    crate::cmd::list::filter_records(&mut records, opts)?;

    let mut inventory = HashMap::new();
    'control: loop {
        anyhow::ensure!(zones.len() > 0, "no zones to build inventory from");
        let mut selection: Option<usize> = None;
        while selection.is_none() || selection.is_some_and(|i| *i > zones.len())
        {
            for (i, zone) in zones.iter().enumerate() {
                println!("[{}] {}: {}", i + 1, zone.name, zone.id);
            }
            match scanner.prompt("(1/2) Choose a zone").await {
                Ok(Some(input)) => selection = input.parse::<usize>().ok(),
                _ => break 'control,
            }
        }
        let zone = &zones[selection.unwrap() - 1];
        let records = records
            .iter()
            .filter(|r| r.zone_id == zone.id)
            .collect::<Vec<&Record>>();
        if records.len() > 0 {
            selection = None;
            while selection.is_none()
                || selection.is_some_and(|i| *i > records.len())
            {
                for (i, record) in records.iter().enumerate() {
                    println!("[{}] {}: {}", i + 1, record.name, record.id);
                }
                match scanner.prompt("(2/2) Choose a record").await {
                    Ok(Some(input)) => selection = input.parse::<usize>().ok(),
                    _ => break 'control,
                }
            }
            let record = &records[selection.unwrap() - 1];
            let key = zone.id.clone();
            let inventory_zone = inventory
                .entry(key)
                .or_insert_with(|| InventoryZone(Some(Vec::new())));
            inventory_zone
                .0
                .as_mut()
                .unwrap()
                .push(InventoryRecord(record.id.clone()));
            println!("Added {}: {}\n", record.name, record.id);
        }
    }
    let inventory = Inventory(Some(inventory));
    Ok(inventory)
}
