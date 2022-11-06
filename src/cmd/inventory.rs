use crate::{
    cloudfare::{self, endpoints::update_record, models::Record},
    config::models::{ConfigOpts, ConfigOptsInventory},
    inventory::models::Inventory,
    inventory::DEFAULT_INVENTORY_PATH,
    io::{self, fs, Scanner},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{
    collections::HashSet,
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
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
    /// Fix erroneous DNS records on an interval.
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
            InventorySubcommands::Commit => commit(&opts).await,
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
    anyhow::ensure!(zones.len() > 0, "no zones to build inventory from");

    let mut records = cloudfare::endpoints::records(&zones, &token).await?;
    crate::cmd::list::filter_records(&mut records, opts)?;
    anyhow::ensure!(records.len() > 0, "no records to build inventory from");

    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);

    // Capture user input to build inventory map
    let mut inventory = Inventory::new();
    'control: loop {
        let zone_idx = 'zone: loop {
            // Print zone options
            for (i, zone) in zones.iter().enumerate() {
                println!("[{}] {}", i + 1, zone);
            }
            // Get zone choice
            if let Some(idx) = scanner
                .prompt("(Step 1 of 2) Choose a zone")
                .await?
                .map(|s| s.parse::<usize>().ok())
                .flatten()
            {
                if idx > 0 && idx <= zones.len() {
                    break idx;
                } else {
                    continue 'zone;
                }
            }
        };
        let selected_zone = &zones[zone_idx - 1];
        let zone_records = records
            .iter()
            .filter(|r| r.zone_id == selected_zone.id)
            .collect::<Vec<&Record>>();

        if zone_records.len() > 0 {
            let record_idx = 'record: loop {
                for (i, record) in zone_records.iter().enumerate() {
                    println!("[{}] {}", i + 1, record);
                }
                if let Some(idx) = scanner
                    .prompt("(Step 2 of 2) Choose a record")
                    .await?
                    .map(|s| s.parse::<usize>().ok())
                    .flatten()
                {
                    if idx > 0 && idx <= zone_records.len() {
                        break idx;
                    } else {
                        continue 'record;
                    }
                }
            };
            let selected_record = &zone_records[record_idx - 1];

            let zone_id = selected_zone.id.clone();
            let record_id = selected_record.id.clone();
            inventory.insert(zone_id, record_id);
            println!("\n✅ Added '{}'.", selected_record.name);
        } else {
            println!("\n❌ No records for this zone.")
        }

        let finished = 'finished: loop {
            match scanner.prompt("Add another record? [Y/n]").await? {
                Some(input) => match input.to_lowercase().as_str() {
                    "y" | "yes" => break false,
                    "n" | "no" => break true,
                    _ => continue 'finished,
                },
                None => break false,
            }
        };
        if finished {
            break 'control;
        }
    }

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
        .flatten()
        .unwrap_or(DEFAULT_INVENTORY_PATH.into());
    let inventory = Inventory::from_file(inventory_path).await?;
    if inventory.is_empty() {
        println!("Inventory file is empty.");
    } else {
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
    }
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
        .flatten()
        .unwrap_or(DEFAULT_INVENTORY_PATH.into());
    let inventory = Inventory::from_file(inventory_path).await?;

    // Check records
    println!("Checking Cloudfare resources...");
    let ipv4 = public_ip::addr_v4().await;
    let ipv6 = public_ip::addr_v6().await;
    let (good, bad, invalid) =
        check_records(token, &inventory, ipv4, ipv6).await?;

    // Print records
    for cf_record in &good {
        println!("MATCH: {} ({})", cf_record.name, cf_record.id);
    }
    for cf_record in &bad {
        println!(
            "MISMATCH: {} ({}) => {}",
            cf_record.name, cf_record.id, cf_record.content
        );
    }
    for (inv_zone, inv_record) in &invalid {
        println!("INVALID: {} | {}", inv_zone, inv_record);
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
        .flatten()
        .unwrap_or(DEFAULT_INVENTORY_PATH.into());
    let mut inventory = Inventory::from_file(&inventory_path).await?;

    // Check records
    println!("Checking Cloudfare resources...");
    let ipv4 = public_ip::addr_v4().await;
    let ipv6 = public_ip::addr_v6().await;
    let (_good, mut bad, mut invalid) =
        check_records(&token, &inventory, ipv4.clone(), ipv6.clone()).await?;

    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);

    // Print records
    if bad.len() > 0 {
        // Print bad records
        for cf_record in &bad {
            println!(
                "MISMATCH: {} ({}) => {}",
                cf_record.name, cf_record.id, cf_record.content
            );
        }
        // Ask to fix records
        let fix = 'fix: loop {
            match scanner
                .prompt(format!("Fix {} bad records? [Y/n]", bad.len()))
                .await?
            {
                Some(input) => match input.to_lowercase().as_str() {
                    "y" | "yes" => break true,
                    "n" | "no" => break false,
                    _ => continue 'fix,
                },
                None => break true,
            }
        };
        // Fix records
        let mut fixed = HashSet::new();
        if fix {
            for cf_record in &bad {
                match cf_record.record_type.as_str() {
                    "A" => match ipv4 {
                        Some(ip) => {
                            update_record(
                                token.clone(),
                                cf_record.zone_id.clone(),
                                cf_record.id.clone(),
                                ip,
                            )
                            .await?
                        }
                        None => anyhow::bail!("no discovered IPv4 address needed to patch A record"),
                    },
                    "AAAA" => match ipv6 {
                        Some(ip) => update_record(
                                token.clone(),
                                cf_record.zone_id.clone(),
                                cf_record.id.clone(),
                                ip,
                            )
                            .await?,
                        None => anyhow::bail!("no discovered IPv6 address needed to patch AAAA record"),
                    },
                    _ => unimplemented!(
                            "unexpected record type: {}",
                            cf_record.record_type
                        ),
                };
                fixed.insert(cf_record.id.clone());
            }
        }
        bad.retain_mut(|r| !fixed.contains(&r.id));
    }

    if invalid.len() > 0 {
        // Print invalid records
        for (inv_zone, inv_record) in &invalid {
            println!("INVALID: {} | {}", inv_zone, inv_record);
        }
        // Ask to prune records
        let prune = 'control: loop {
            match scanner
                .prompt(format!(
                    "Prune {} invalid records? [Y/n]",
                    invalid.len()
                ))
                .await?
            {
                Some(input) => match input.to_lowercase().as_str() {
                    "n" | "no" => break false,
                    "y" | "yes" => break true,
                    _ => continue 'control,
                },
                None => break true,
            }
        };
        // Prune
        let mut pruned = HashSet::new();
        if prune {
            for (zone_id, record_id) in invalid.iter() {
                let removed = inventory
                    .remove(zone_id.to_owned(), record_id.to_owned())?;
                if removed {
                    pruned.insert((zone_id.clone(), record_id.clone()));
                }
            }
            invalid.retain_mut(|(z, r)| {
                !pruned.contains(&(z.to_owned(), r.to_owned()))
            });
            fs::remove_force(&inventory_path).await?;
            fs::save_yaml(&inventory, &inventory_path).await?;
        }
    }

    // Print summary
    if bad.len() == 0 && invalid.len() == 0 {
        println!("✅ No bad or invalid records.");
    } else {
        println!(
            "❌ {} bad, {} invalid records remain.",
            bad.len(),
            invalid.len()
        );
    }
    Ok(())
}

pub async fn check_records(
    token: impl Display,
    inventory: &Inventory,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
) -> Result<(Vec<Record>, Vec<Record>, Vec<(String, String)>)> {
    let zones = cloudfare::endpoints::zones(token.to_string()).await?;
    let records =
        cloudfare::endpoints::records(&zones, token.to_string()).await?;

    // Check and collect records
    let (mut good, mut bad, mut invalid) = (vec![], vec![], vec![]);
    for (inv_zone, inv_records) in inventory.clone().into_iter() {
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
