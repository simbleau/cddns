use crate::{
    cloudflare::{self, endpoints::update_record, models::Record},
    config::models::{
        ConfigOpts, ConfigOptsCommit, ConfigOptsInventory, ConfigOptsWatch,
    },
    inventory::{
        default_inventory_path,
        models::{Inventory, InventoryData},
    },
    io::{self, encoding::InventoryPostProcessor, Scanner},
};
use anyhow::{ensure, Context, Result};
use clap::{Args, Subcommand};
use std::{
    collections::HashSet,
    fmt::Debug,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    vec,
};
use tokio::time::{self, Duration, MissedTickBehavior};
use tracing::{debug, error, info, warn};

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
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let cli_cfg = ConfigOpts {
            inventory: Some(self.cfg),
            ..Default::default()
        };
        let opts = ConfigOpts::full(config, Some(cli_cfg))?;

        match self.action {
            InventorySubcommands::Build => build(&opts).await,
            InventorySubcommands::Show => show(&opts).await,
            InventorySubcommands::Check => check(&opts).await.map(|_| ()),
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

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn build(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;

    // Get zones and records to build inventory from
    info!("retrieving cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    crate::cmd::list::retain_zones(&mut zones, opts)?;
    ensure!(!zones.is_empty(), "no zones to build inventory from");
    let mut records = cloudflare::endpoints::records(&zones, &token).await?;
    crate::cmd::list::retain_records(&mut records, opts)?;

    let mut data = InventoryData(None);
    let mut scanner = Scanner::new(tokio::runtime::Handle::current());
    if records.is_empty() {
        warn!("there are no records visible to this token, but you may save an empty inventory");
    } else {
        records.sort_by_key(|r| r.id.to_owned());
        // Capture user input to build inventory map
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
                let selected_record = 'record: loop {
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
                            let r_idx = records
                                .binary_search_by_key(
                                    &zone_records[idx - 1].id,
                                    |r| r.id.clone(),
                                )
                                .ok()
                                .with_context(|| {
                                    format!("option {} not found", idx)
                                })?;
                            records.remove(r_idx); // Remove for next query
                            break &records[r_idx];
                        } else {
                            continue 'record;
                        }
                    }
                };
                data.insert(&selected_zone.id, &selected_record.id);
                println!("✅ Added '{}'.", selected_record.name);
            } else {
                println!("❌ No records for this zone.")
            }

            let add_more = scanner
                .prompt_yes_or_no("Add another record?", "Y/n")
                .await?
                .unwrap_or(true);
            if !add_more {
                break 'control;
            }
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
    io::fs::remove_interactive(&path, &mut scanner).await?;

    info!("saving inventory file...");
    // Best-effort attempt to post-process comments on inventory.
    let pp = InventoryPostProcessor::try_init(opts).await.ok();
    if pp.is_none() {
        warn!("failed to initialize post-processor")
    }
    Inventory::builder()
        .path(path)
        .with_data(data)
        .build()?
        .save(pp)
        .await?;

    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn show(opts: &ConfigOpts) -> Result<()> {
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    if inventory.data.is_empty() {
        warn!("inventory is empty");
    } else {
        println!("{}", inventory);
    }
    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn check(opts: &ConfigOpts) -> Result<CheckResult> {
    // Get inventory
    info!("reading inventory...");
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    // End early if inventory is empty
    if inventory.data.is_empty() {
        warn!("inventory is empty");
        return Ok(CheckResult::default());
    }

    // Get cloudflare records and zones
    info!("retrieving cloudflare resources...");
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;
    let zones = cloudflare::endpoints::zones(token.to_string()).await?;
    let records =
        cloudflare::endpoints::records(&zones, token.to_string()).await?;

    // Match zones and records
    info!("matching records...");
    let mut ipv4: Option<Ipv4Addr> = None;
    let mut ipv6: Option<Ipv6Addr> = None;
    let (mut matches, mut mismatches, mut invalid) = (vec![], vec![], vec![]);
    for (ref inv_zone, ref inv_records) in inventory.data.into_iter() {
        for inv_record in inv_records {
            let cf_record = records.iter().find(|r| {
                (r.zone_id == *inv_zone || r.zone_name == *inv_zone)
                    && (r.id == *inv_record || r.name == *inv_record)
            });
            match cf_record {
                Some(cf_record) => {
                    let ip = match cf_record.record_type.as_str() {
                        "A" => ipv4
                            .get_or_insert({
                                info!("resolving ipv4...");
                                public_ip::addr_v4()
                                    .await
                                    .context("could not resolve ipv4 address")?
                            })
                            .to_string(),
                        "AAAA" => ipv6
                            .get_or_insert({
                                info!("resolving ipv6...");
                                public_ip::addr_v6()
                                    .await
                                    .context("could not resolve ipv6 address")?
                            })
                            .to_string(),
                        _ => unimplemented!(),
                    };
                    if cf_record.content == ip {
                        // IP Match
                        info!("match: {}", cf_record);
                        matches.push(cf_record.clone());
                    } else {
                        // IP mismatch
                        warn!("mismatch: {}", cf_record);
                        mismatches.push(cf_record.clone());
                    }
                }
                None => {
                    // Invalid record, no match on zone and record
                    warn!("invalid: {} | {}", inv_zone, inv_record);
                    invalid.push((inv_zone.clone(), inv_record.clone()));
                }
            }
        }
    }

    // Log summary
    info!(
        "✅ {} matched, ❌ {} mismatched, ❓ {} invalid",
        matches.len(),
        mismatches.len(),
        invalid.len()
    );
    if !mismatches.is_empty() || !invalid.is_empty() {
        warn!("mismatching or invalid records exist");
    }
    Ok(CheckResult {
        matches,
        mismatches,
        invalid,
    })
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn commit(opts: &ConfigOpts) -> Result<()> {
    let CheckResult {
        matches,
        mut mismatches,
        mut invalid,
    } = check(opts).await?;

    // Fix mismatched records
    if !mismatches.is_empty() {
        let fixed_record_ids = update(opts, &mismatches)
            .await
            .context("error updating mismatched records")?;
        mismatches.retain_mut(|r| !fixed_record_ids.contains(&r.id));
    }

    // Prune invalid records
    if !invalid.is_empty() {
        let new_inventory = prune(opts, &invalid).await?;
        invalid.retain(|(z, r)| new_inventory.data.contains(z, r));
    }

    // Log summary
    info!(
        "✅ {} matched, ❌ {} mismatched, ❓ {} invalid",
        matches.len(),
        mismatches.len(),
        invalid.len()
    );
    if !mismatches.is_empty() || !invalid.is_empty() {
        warn!("mismatching or invalid records remain");
    }

    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn watch(opts: &ConfigOpts) -> Result<()> {
    // Override force flag with true; watch is non-interactive
    let opts = opts.clone().merge(ConfigOpts {
        commit: Some(ConfigOptsCommit { force: true }),
        ..Default::default()
    });

    // Get watch interval
    let interval = Duration::from_millis(
        opts.watch
            .as_ref()
            .and_then(|opts| opts.interval)
            .or(ConfigOptsWatch::default().interval)
            .context("no default interval")?,
    );
    debug!("interval: {}ms", interval.as_millis());

    if interval.is_zero() {
        loop {
            if let Err(e) = commit(&opts).await {
                error!("{:?}", e);
            }
        }
    } else {
        let mut timer = time::interval(interval);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            debug!("awoken");
            if let Err(e) = commit(&opts).await {
                error!("{:?}", e);
            }
            debug!("sleeping...");
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CheckResult {
    matches: Vec<Record>,
    mismatches: Vec<Record>,
    invalid: Vec<(String, String)>,
}

/// Update a list of mismatching records, returning those ids which were
/// successfully updated.
pub async fn update(
    opts: &ConfigOpts,
    mismatches: &Vec<Record>,
) -> Result<HashSet<String>> {
    // Track fixed records
    let mut updated_ids = HashSet::new();
    // Fix mismatched records
    if !mismatches.is_empty() {
        let force = opts
            .commit
            .as_ref()
            .map(|opts| opts.force)
            .unwrap_or(ConfigOptsCommit::default().force);
        debug!("force update: {}", force);

        // Ask to fix records
        let fix = force || {
            let mut scanner = Scanner::new(tokio::runtime::Handle::current());
            scanner
                .prompt_yes_or_no(
                    format!("Update {} mismatched records?", mismatches.len()),
                    "Y/n",
                )
                .await?
                .unwrap_or(true)
        };
        if fix {
            info!("updating records...");
            let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;
            let mut ipv4: Option<Ipv4Addr> = None;
            let mut ipv6: Option<Ipv6Addr> = None;
            for cf_record in mismatches.iter() {
                let updated = match cf_record.record_type.as_str() {
                    "A" => {
                        update_record(
                            &token,
                            &cf_record.zone_id,
                            &cf_record.id,
                            ipv4.get_or_insert({
                                info!("resolving ipv4...");
                                public_ip::addr_v4()
                                    .await
                                    .context("could not resolve ipv4 address")?
                            })
                            .to_string(),
                        )
                        .await
                    }
                    "AAAA" => {
                        update_record(
                            &token,
                            &cf_record.zone_id,
                            &cf_record.id,
                            ipv6.get_or_insert({
                                info!("resolving ipv6...");
                                public_ip::addr_v6()
                                    .await
                                    .context("could not resolve ipv6 address")?
                            })
                            .to_string(),
                        )
                        .await
                    }
                    _ => unimplemented!(),
                };
                if updated.is_ok() {
                    debug!("updated '{}'", &cf_record.id);
                    updated_ids.insert(cf_record.id.clone());
                } else {
                    error!("unsuccessful update of '{}'", &cf_record.id)
                }
            }
        }
    }
    Ok(updated_ids)
}

/// Prune invalid records, returning the resulting inventory.
pub async fn prune(
    opts: &ConfigOpts,
    invalid: &Vec<(String, String)>,
) -> Result<Inventory> {
    // Get inventory
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let mut inventory = Inventory::from_file(&inventory_path).await?;

    // Prune invalid records
    if !invalid.is_empty() {
        let force = opts
            .commit
            .as_ref()
            .map(|opts| opts.force)
            .unwrap_or(ConfigOptsCommit::default().force);
        debug!("force prune: {}", force);

        // Ask to prune records
        let prune = force || {
            let mut scanner = Scanner::new(tokio::runtime::Handle::current());
            scanner
                .prompt_yes_or_no(
                    format!("Prune {} invalid records?", invalid.len()),
                    "Y/n",
                )
                .await?
                .unwrap_or(true)
        };
        // Prune
        if prune {
            info!("removing invalid records...");
            for (zone_id, record_id) in invalid.iter() {
                let removed = inventory.data.remove(zone_id, record_id);
                if let Ok(true) = removed {
                    debug!("pruned '{} | {}'", &zone_id, &record_id);
                } else {
                    error!("could not remove '{} | {}'", &zone_id, &record_id);
                }
            }
        }
        info!("updating inventory file...");
        // Best-effort attempt to post-process comments on inventory.
        let pp = InventoryPostProcessor::try_init(opts).await.ok();
        if pp.is_none() {
            warn!("failed to initialize post-processor")
        }
        inventory.save(pp).await?;
    }

    Ok(inventory)
}
