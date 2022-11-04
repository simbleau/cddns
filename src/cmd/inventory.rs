use crate::{
    cloudfare::{self, models::Record},
    config::models::{ConfigOpts, ConfigOptsInventory},
    inventory::models::{Inventory, InventoryRecord, InventoryZone},
    inventory::DEFAULT_INVENTORY_PATH,
    io::{self, Scanner},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    vec,
};

/// Build or manage your DNS record inventory.
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
            InventorySubcommands::Build => build(&opts).await,
            InventorySubcommands::Show => show(&opts).await,
            InventorySubcommands::Check => check(&opts).await,
            InventorySubcommands::Commit => todo!(),
            InventorySubcommands::Watch => todo!(),
        }
    }
}

async fn build(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .map(|opts| opts.token.clone())
        .flatten()
        .context("no token was provided")?;

    // Get zones and records to build inventory from
    println!("Retrieving Cloudfare resources...");
    let mut zones = cloudfare::endpoints::zones(&token).await?;
    crate::cmd::list::filter_zones(&mut zones, opts)?;
    let mut records = cloudfare::endpoints::records(&zones, &token).await?;
    crate::cmd::list::filter_records(&mut records, opts)?;

    // Control user input to build inventory map
    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);
    let mut inventory_map = HashMap::new();
    'control: loop {
        anyhow::ensure!(zones.len() > 0, "no zones to build inventory from");
        let mut selection: Option<usize> = None;
        while selection.is_none() || selection.is_some_and(|i| *i > zones.len())
        {
            for (i, zone) in zones.iter().enumerate() {
                println!("[{}] {}", i + 1, zone);
            }
            match scanner.prompt("🌎 Choose a zone").await? {
                Some(input) => selection = input.parse::<usize>().ok(),
                None => selection = None,
            }
        }
        let selected_zone = &zones[selection.unwrap() - 1];
        let records = records
            .iter()
            .filter(|r| r.zone_id == selected_zone.id)
            .collect::<Vec<&Record>>();
        if records.len() > 0 {
            selection = None;
            while selection.is_none()
                || selection.is_some_and(|i| *i > records.len())
            {
                for (i, record) in records.iter().enumerate() {
                    println!("[{}] {}", i + 1, record);
                }
                match scanner.prompt("🌎 Choose a record").await? {
                    Some(input) => selection = input.parse::<usize>().ok(),
                    None => selection = None,
                }
            }
            let record = &records[selection.unwrap() - 1];
            let key = selected_zone.id.clone();
            let inventory_zone = inventory_map
                .entry(key)
                .or_insert_with(|| InventoryZone(Some(HashSet::new())));
            inventory_zone
                .0
                .as_mut()
                .unwrap()
                .insert(InventoryRecord(record.id.clone()));
            println!("\n✅ Added '{}'.", record.name);
        } else {
            println!("\n❌ No records for this zone.")
        }

        if let Some(input) = scanner.prompt("Add another record? [Y/n]").await?
        {
            if matches!(input.to_lowercase().as_str(), "n" | "no") {
                break 'control;
            }
        }
    }
    let inventory = Inventory(Some(inventory_map));

    // Save
    let path = scanner
        .prompt_path_or(
            format!("💾 Save location [default: {}]", DEFAULT_INVENTORY_PATH),
            DEFAULT_INVENTORY_PATH.into(),
        )
        .await
        .map(|p| match p.extension() {
            Some(_) => p,
            None => p.with_extension("yaml"),
        })?;
    if path.exists() {
        io::fs::remove_interactive(&path, &mut scanner).await?;
    }
    io::fs::save_yaml(&inventory, path).await?;
    println!("✅ Saved");

    Ok(())
}

async fn show(opts: &ConfigOpts) -> Result<()> {
    let inventory_path = opts
        .inventory
        .as_ref()
        .map(|opts| opts.path.clone())
        .flatten();
    let inventory = Inventory::from_file(inventory_path)?;
    let pretty_print = inventory
        .into_iter()
        .map(|(zone, records)| {
            format!(
                "{}:{}",
                zone,
                records
                    .into_iter()
                    .map(|r| format!("\n  - {}", r))
                    .collect::<String>()
            )
        })
        .intersperse("\n---\n".to_string())
        .collect::<String>();
    println!("{}", pretty_print);
    Ok(())
}

async fn check(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .map(|opts| opts.token.clone())
        .flatten()
        .context("no token was provided")?;

    // Get inventory
    let inventory_path = opts
        .inventory
        .as_ref()
        .map(|opts| opts.path.clone())
        .flatten();
    let inventory = Inventory::from_file(inventory_path)?;

    // Check records
    println!("Checking Cloudfare resources...");
    let (good, bad, invalid) = check_records(token, inventory).await?;

    // Print records
    for cf_record in &good {
        println!("✅ MATCH: {} ({})", cf_record.name, cf_record.id);
    }
    for cf_record in &bad {
        println!(
            "❌ MISMATCH: {} ({}) => {}",
            cf_record.name, cf_record.id, cf_record.content
        );
    }
    for (inv_zone, inv_record) in &invalid {
        println!("❓ INVALID: {} | {}", inv_zone, inv_record);
    }

    // Print summary
    println!(
        "✅ {} GOOD, ❌ {} BAD, ❓ {} INVALID",
        good.len(),
        bad.len(),
        invalid.len()
    );

    Ok(())
}

async fn commit(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .map(|opts| opts.token.clone())
        .flatten()
        .context("no token was provided")?;

    // Get inventory
    let inventory_path = opts
        .inventory
        .as_ref()
        .map(|opts| opts.path.clone())
        .flatten();
    let inventory = Inventory::from_file(inventory_path)?;

    // Check records
    println!("Checking Cloudfare resources...");
    let (good, bad, invalid) = check_records(token, inventory).await?;

    // Print records
    for cf_record in &bad {
        println!(
            "❌ MISMATCH: {} ({}) => {}",
            cf_record.name, cf_record.id, cf_record.content
        );
    }
    // Print summary
    println!("❌ {} BAD", bad.len());

    // Ask to fix records
    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);
    'control: loop {
        if let Some(input) = scanner.prompt("🔨 Fix bad records? [Y/n]").await?
        {
            match input.to_lowercase().as_str() {
                "n" | "no" => break 'control,
                "y" | "yes" => {
                    todo!("Fix records");
                    // Print summary
                    println!("✅ {} CHANGED", bad.len());
                }
                _ => continue 'control,
            }
        }
    }

    for (inv_zone, inv_record) in &invalid {
        println!("❓ INVALID: {} | {}", inv_zone, inv_record);
    }
    'control: loop {
        if let Some(input) =
            scanner.prompt("🗑️ Prune invalid records? [Y/n]").await?
        {
            match input.to_lowercase().as_str() {
                "n" | "no" => break 'control,
                "y" | "yes" => {
                    todo!("Remove invalid records");
                    // Print summary
                    println!("✅ {} REMOVED", invalid.len());
                }
                _ => continue 'control,
            }
        }
    }

    Ok(())
}

pub async fn check_records(
    token: String,
    inventory: Inventory,
) -> Result<(Vec<Record>, Vec<Record>, Vec<(String, String)>)> {
    // Get public IPs
    let ipv4 = public_ip::addr_v4().await;
    let ipv6 = public_ip::addr_v6().await;

    let zones = cloudfare::endpoints::zones(&token).await?;
    let records = cloudfare::endpoints::records(&zones, &token).await?;

    // Check and collect records
    let (mut good, mut bad, mut invalid) = (vec![], vec![], vec![]);
    for (inv_zone, inv_records) in inventory.into_iter() {
        for inv_record in inv_records {
            let cf_record = records.iter().find(|r| {
                (r.zone_id == inv_zone || r.zone_name == inv_zone)
                    && (r.id == inv_record || r.name == inv_record)
            });
            match cf_record {
                Some(cf_record) => {
                    let ip = match cf_record.record_type.as_str() {
                        "A" => ipv4.map(|ip| ip.to_string()),
                        "AAAA" => ipv6.map(|ip| ip.to_string()),
                        _ => unimplemented!(
                            "unexpected record type: {}",
                            cf_record.record_type
                        ),
                    };
                    if let Some(ref ip) = ip {
                        if &cf_record.content == ip {
                            // IP Match
                            good.push(cf_record.clone());
                        } else {
                            // IP mismatch
                            bad.push(cf_record.clone());
                        }
                    } else {
                        anyhow::bail!(
                            "error no address comparable for {} record",
                            cf_record.record_type
                        );
                    }
                }
                None => {
                    // Invalid record, no match on zone and record
                    invalid.push((inv_zone.clone(), inv_record.clone()));
                }
            }
        }
    }

    Ok((good, bad, invalid))
}
