use crate::{
    cloudflare::{self, endpoints::update_record, models::Record},
    config::models::{
        ConfigOpts, ConfigOptsCommit, ConfigOptsInventory, ConfigOptsWatch,
    },
    inventory::{default_inventory_path, models::Inventory},
    io::{self, Scanner},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{
    collections::HashSet,
    fmt::Display,
    net::{Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    vec,
};
use tokio::time::{self, Duration, MissedTickBehavior};

/// Build or manage your DNS record inventory.
#[derive(Debug, Args)]
#[clap(name = "inventory")]
pub struct InventoryCmd {
    #[clap(subcommand)]
    action: InventorySubcommands,
    #[clap(flatten)]
    pub cfg: ConfigOptsInventory,
}

/// Fix erroneous DNS records once.
#[derive(Debug, Clone, Args)]
#[clap(name = "commit")]
pub struct InventoryCommitCmd {
    #[clap(flatten)]
    pub cfg: ConfigOptsCommit,
}

/// Fix erroneous DNS records on an interval.
#[derive(Debug, Clone, Args)]
#[clap(name = "commit")]
pub struct InventoryWatchCmd {
    #[clap(flatten)]
    pub cfg: ConfigOptsWatch,
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
    Commit(ConfigOptsCommit),
    /// Fix erroneous DNS records on an interval.
    Watch(ConfigOptsWatch),
}

impl InventoryCmd {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let cli_cfg = ConfigOpts {
            inventory: Some(self.cfg),
            ..Default::default()
        };
        let opts = ConfigOpts::full(config, Some(cli_cfg))?;

        match self.action {
            InventorySubcommands::Build => build(&opts).await,
            InventorySubcommands::Show => show(&opts).await,
            InventorySubcommands::Check => check(&opts).await,
            InventorySubcommands::Commit(cfg) => {
                let cli_cfg = ConfigOpts {
                    commit: Some(cfg),
                    ..Default::default()
                };
                commit(&opts.merge(cli_cfg)).await
            }
            InventorySubcommands::Watch(cfg) => {
                let cli_cfg = ConfigOpts {
                    watch: Some(cfg),
                    ..Default::default()
                };
                watch(&opts.merge(cli_cfg)).await
            }
        }
    }
}

async fn build(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get zones and records to build inventory from
    println!("Retrieving Cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    crate::cmd::list::filter_zones(&mut zones, opts)?;
    anyhow::ensure!(!zones.is_empty(), "no zones to build inventory from");

    let mut records = cloudflare::endpoints::records(&zones, &token).await?;
    crate::cmd::list::filter_records(&mut records, opts)?;
    anyhow::ensure!(!records.is_empty(), "no records to build inventory from");

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
                .prompt_t::<usize>("(Step 1 of 2) Choose a zone", "number")
                .await?
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

        if !zone_records.is_empty() {
            let record_idx = 'record: loop {
                for (i, record) in zone_records.iter().enumerate() {
                    println!("[{}] {}", i + 1, record);
                }
                if let Some(idx) = scanner
                    .prompt_t::<usize>(
                        "(Step 2 of 2) Choose a record",
                        "number",
                    )
                    .await?
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
            match scanner.prompt("Add another record?", "Y/n").await? {
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
        .prompt_t::<PathBuf>(
            format!(
                "Save location, default: `{}`",
                default_inventory_path().display()
            ),
            "path",
        )
        .await?
        .map(|p| match p.extension() {
            Some(_) => p,
            None => p.with_extension("yaml"),
        })
        .unwrap_or_else(default_inventory_path);
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
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
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
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get inventory
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    // Check records
    println!("Checking Cloudflare resources...");
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
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get inventory
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let mut inventory = Inventory::from_file(&inventory_path).await?;

    let force = opts
        .commit
        .as_ref()
        .map(|opts| opts.force)
        .unwrap_or(ConfigOptsCommit::default().force);

    // Check records
    println!("Checking Cloudflare resources...");
    let ipv4 = public_ip::addr_v4().await;
    let ipv6 = public_ip::addr_v6().await;
    let (_good, mut bad, mut invalid) =
        check_records(&token, &inventory, ipv4, ipv6).await?;

    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);

    // Print records
    if !bad.is_empty() {
        // Print bad records
        for cf_record in &bad {
            println!(
                "MISMATCH: {} ({}) => {}",
                cf_record.name, cf_record.id, cf_record.content
            );
        }
        // Ask to fix records
        let fix = force
            || scanner
                .prompt_yes_or_no(
                    format!("Fix {} bad records?", bad.len()),
                    "Y/n",
                )
                .await?
                .unwrap_or(true);
        // Fix records
        let mut fixed = HashSet::new();
        if fix {
            for cf_record in &bad {
                if match cf_record.record_type.as_str() {
                    "A" => match ipv4 {
                        Some(ip) => {
                            update_record(
                                token.clone(),
                                cf_record.zone_id.clone(),
                                cf_record.id.clone(),
                                ip,
                            )
                            .await
                        }
                        None => Err(anyhow::anyhow!("no discovered IPv4 address needed to patch A record")),
                    },
                    "AAAA" => match ipv6 {
                        Some(ip) => update_record(
                                token.clone(),
                                cf_record.zone_id.clone(),
                                cf_record.id.clone(),
                                ip,
                            )
                            .await,
                        None => Err(anyhow::anyhow!("no discovered IPv6 address needed to patch AAAA record")),
                    },
                    _ => unimplemented!(
                            "unexpected record type: {}",
                            cf_record.record_type
                        ),
                }.is_ok() {
                    fixed.insert(cf_record.id.clone());
                }
            }
        }
        bad.retain_mut(|r| !fixed.contains(&r.id));
    }

    if !invalid.is_empty() {
        // Print invalid records
        for (inv_zone, inv_record) in &invalid {
            println!("INVALID: {} | {}", inv_zone, inv_record);
        }
        // Ask to prune records
        let prune = force
            || scanner
                .prompt_yes_or_no(
                    format!("Prune {} invalid records?", invalid.len()),
                    "Y/n",
                )
                .await?
                .unwrap_or(true);
        // Prune
        let mut pruned = HashSet::new();
        if prune {
            for (zone_id, record_id) in invalid.iter() {
                let removed =
                    inventory.remove(zone_id.to_owned(), record_id.to_owned());
                if let Ok(true) = removed {
                    pruned.insert((zone_id.clone(), record_id.clone()));
                }
            }
            invalid.retain_mut(|(z, r)| {
                !pruned.contains(&(z.to_owned(), r.to_owned()))
            });
            inventory.save(inventory_path).await?;
        }
    }

    // Print summary
    if bad.is_empty() && invalid.is_empty() {
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

pub async fn watch(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get inventory
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let mut inventory = Inventory::from_file(&inventory_path).await?;

    // Get watch interval
    let interval = Duration::from_millis(
        opts.watch
            .as_ref()
            .and_then(|opts| opts.interval)
            .unwrap_or_else(|| ConfigOptsWatch::default().interval.unwrap()),
    );

    if interval.is_zero() {
        loop {
            if let Err(e) =
                __watch(&token, &mut inventory, &inventory_path).await
            {
                println!("Error: {:?}", e);
            }
        }
    } else {
        let mut timer = time::interval(interval);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            if let Err(e) =
                __watch(&token, &mut inventory, &inventory_path).await
            {
                println!("Error: {:?}", e);
            }
        }
    }
}

pub async fn check_records(
    token: impl Display,
    inventory: &Inventory,
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
) -> Result<(Vec<Record>, Vec<Record>, Vec<(String, String)>)> {
    let zones = cloudflare::endpoints::zones(token.to_string()).await?;
    let records =
        cloudflare::endpoints::records(&zones, token.to_string()).await?;

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
                            "no address comparable for {} record",
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

/// This would be fantastic as an async closure when that becomes stabalized.
/// For now, this is a helper to perform the commits without interaction.
async fn __watch<P>(
    token: impl Display,
    inventory: &mut Inventory,
    inventory_path: P,
) -> Result<()>
where
    P: AsRef<Path>,
{
    let token = token.to_string();

    // Check records
    println!("Checking Cloudflare resources...");
    let ipv4 = public_ip::addr_v4().await;
    let ipv6 = public_ip::addr_v6().await;
    let (_good, mut bad, mut invalid) =
        check_records(&token, inventory, ipv4, ipv6).await?;

    // Print records
    if !bad.is_empty() {
        // Fix records
        let mut fixed = HashSet::new();
        for cf_record in &bad {
            println!(
                "MISMATCH: {} ({}) => {}",
                cf_record.name, cf_record.id, cf_record.content
            );
            if match cf_record.record_type.as_str() {
                    "A" => match ipv4 {
                        Some(ip) => {
                            update_record(
                                token.clone(),
                                cf_record.zone_id.clone(),
                                cf_record.id.clone(),
                                ip,
                            )
                            .await
                        }
                        None => Err(anyhow::anyhow!("no discovered IPv4 address needed to patch A record")),
                    },
                    "AAAA" => match ipv6 {
                        Some(ip) => update_record(
                                token.clone(),
                                cf_record.zone_id.clone(),
                                cf_record.id.clone(),
                                ip,
                            )
                            .await,
                        None => Err(anyhow::anyhow!("no discovered IPv6 address needed to patch AAAA record")),
                    },
                    _ => unimplemented!(
                            "unexpected record type: {}",
                            cf_record.record_type
                        ),
                }.is_ok() {
                    fixed.insert(cf_record.id.clone());
                }
        }
        bad.retain_mut(|r| !fixed.contains(&r.id));
    }

    // Prune
    let mut pruned = HashSet::new();
    if !invalid.is_empty() {
        // Print invalid records
        for (inv_zone, inv_record) in &invalid {
            println!("INVALID: {} | {}", inv_zone, inv_record);
        }
        for (zone_id, record_id) in invalid.iter() {
            let removed =
                inventory.remove(zone_id.to_owned(), record_id.to_owned());
            if let Ok(true) = removed {
                pruned.insert((zone_id.clone(), record_id.clone()));
            }
        }
        invalid.retain_mut(|(z, r)| {
            !pruned.contains(&(z.to_owned(), r.to_owned()))
        });
        inventory.save(inventory_path).await?;
    }

    // Print summary
    if bad.is_empty() && invalid.is_empty() {
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
