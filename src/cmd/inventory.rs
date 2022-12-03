use crate::{
    cloudflare::{self, endpoints::update_record, models::Record},
    config::models::{
        ConfigOpts, ConfigOptsInventory, ConfigOptsInventoryCommit,
        ConfigOptsInventoryWatch,
    },
    inventory::{
        default_inventory_path,
        models::{Inventory, InventoryData},
    },
    io::{
        self,
        encoding::InventoryPostProcessor,
        scanner::{prompt_t, prompt_yes_or_no},
    },
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{
    collections::HashSet,
    fmt::Debug,
    net::{Ipv4Addr, Ipv6Addr},
    path::PathBuf,
    vec,
};
use tokio::time::{self, Duration, MissedTickBehavior};
use tracing::{debug, error, info, trace, warn};

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
    pub cfg: ConfigOptsInventoryCommit,
}

/// Fix erroneous DNS records on an interval.
#[derive(Debug, Clone, Args)]
#[clap(name = "commit")]
pub struct InventoryWatchCmd {
    #[clap(flatten)]
    pub cfg: ConfigOptsInventoryWatch,
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
    Commit(ConfigOptsInventoryCommit),
    /// Fix erroneous DNS records on an interval.
    Watch(ConfigOptsInventoryWatch),
}

impl InventoryCmd {
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        // Apply CLI configuration layering
        let mut cli_opts =
            ConfigOpts::builder().inventory(Some(self.cfg)).build();
        if let InventorySubcommands::Commit(ref cli_commit) = self.action {
            cli_opts = ConfigOpts::builder()
                .merge(cli_opts)
                .inventory_commit(Some(cli_commit.clone()))
                .build();
        } else if let InventorySubcommands::Watch(ref cli_watch) = self.action {
            cli_opts = ConfigOpts::builder()
                .merge(cli_opts)
                .inventory_watch(Some(cli_watch.clone()))
                .build();
        }
        let opts = ConfigOpts::full(config, Some(cli_opts))?;

        // Run
        match self.action {
            InventorySubcommands::Build => build(&opts).await,
            InventorySubcommands::Show => show(&opts).await,
            InventorySubcommands::Check => check(&opts).await.map(|_| ()),
            InventorySubcommands::Commit(_) => commit(&opts).await,
            InventorySubcommands::Watch(_) => watch(&opts).await,
        }
    }
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn build(opts: &ConfigOpts) -> Result<()> {
    info!("getting ready, please wait...");
    // Get zones and records to build inventory from
    let token = opts
        .verify.token.as_ref()
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;
    trace!("retrieving cloudflare resources...");
    let mut all_zones = cloudflare::endpoints::zones(&token).await?;
    crate::cmd::list::retain_zones(&mut all_zones, opts)?;
    let mut all_records =
        cloudflare::endpoints::records(&all_zones, &token).await?;
    crate::cmd::list::retain_records(&mut all_records, opts)?;

    // Sort by name
    all_zones.sort_by_key(|z| z.name.to_owned());
    all_records.sort_by_key(|r| r.name.to_owned());

    let mut data = InventoryData(None);
    if all_records.is_empty() {
        warn!("there are no records visible to this token, but you may save an empty inventory");
    } else {
        // Capture user input to build inventory map
        'control: loop {
            // Get zone index
            let zone_index = 'zone: loop {
                // Print zone options
                for (i, zone) in all_zones.iter().enumerate() {
                    println!("[{}] {}", i + 1, zone);
                }
                // Get zone choice
                if let Some(idx) =
                    prompt_t::<usize>("(Step 1 of 2) Choose a zone", "number")?
                {
                    if idx > 0 && idx <= all_zones.len() {
                        debug!(input = idx);
                        break idx - 1;
                    } else {
                        warn!("invalid option: {}", idx);
                        continue 'zone;
                    }
                }
            };

            // Filter records
            let record_options = all_records
                .iter()
                .filter(|r| r.zone_id == all_zones[zone_index].id)
                .collect::<Vec<&Record>>();
            if record_options.is_empty() {
                error!("❌ No records for this zone.");
                continue 'control;
            }
            // Get record index
            let record_index = 'record: loop {
                for (i, record) in record_options.iter().enumerate() {
                    println!("[{}] {}", i + 1, record);
                }
                if let Some(idx) = prompt_t::<usize>(
                    "(Step 2 of 2) Choose a record",
                    "number",
                )? {
                    if idx > 0 && idx <= record_options.len() {
                        debug!(input = idx);
                        break all_records
                            .binary_search_by_key(
                                &record_options[idx - 1].name,
                                |r| r.name.clone(),
                            )
                            .ok()
                            .with_context(|| {
                                format!("option {idx} not found")
                            })?;
                    } else {
                        warn!("invalid option: {idx}");
                        continue 'record;
                    }
                }
            };
            // Append record to data
            let selected_zone = &all_zones[zone_index];
            let selected_record = &all_records[record_index];
            data.insert(&selected_zone.id, &selected_record.id);
            println!("✅ Added '{}'.", selected_record.name);

            // Remove for next iteration
            if record_options.len() == 1 {
                all_zones.remove(zone_index);
            }
            all_records.remove(record_index);

            // Prepare next iteration
            if all_zones.is_empty() {
                println!("No records left. Continuing...");
                break 'control;
            } else {
                let add_more = prompt_yes_or_no("Add another record?", "Y/n")?
                    .unwrap_or(true);
                if !add_more {
                    break 'control;
                }
            }
        }
    }

    // Save
    let path = prompt_t::<PathBuf>(
        format!(
            "Save location [default: {}]",
            default_inventory_path().display()
        ),
        "path",
    )?
    .map(|p| match p.extension() {
        Some(_) => p,
        None => p.with_extension("yaml"),
    })
    .unwrap_or_else(default_inventory_path);
    io::fs::remove_interactive(&path).await?;

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
        .path
        .clone()
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    if inventory.data.is_empty() {
        warn!("inventory is empty");
    } else {
        println!("{}", inventory);
    }
    Ok(())
}

pub async fn check(opts: &ConfigOpts) -> Result<CheckResult> {
    info!("checking records, please wait...");
    // Get inventory
    trace!("refreshing inventory...");
    let inventory_path = opts
        .inventory
        .path
        .clone()
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    trace!("retrieving cloudflare resources...");
    // Token is required to fix inventory record.
    let token = opts
        .verify.token.as_ref()
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;

    // End early if inventory is empty
    if inventory.data.is_empty() {
        warn!("inventory is empty");
        return Ok(CheckResult::default());
    }
    // Get cloudflare records and zones
    let zones = cloudflare::endpoints::zones(token.to_string()).await?;
    let records =
        cloudflare::endpoints::records(&zones, token.to_string()).await?;

    // Match zones and records
    trace!("matching records...");
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
                        "A" => {
                            match ipv4 {
                                Some(ip) => ip,
                                None => {
                                    trace!("resolving ipv4...");
                                    let ip = public_ip::addr_v4()
                                        .await
                                        .context("could not resolve public ipv4 needed for A record")?;
                                    ipv4.replace(ip);
                                    ip
                                }
                            }
                        }
                        .to_string(),
                        "AAAA" => {
                            match ipv6 {
                                Some(ip) => ip,
                                None => {
                                    trace!("resolving ipv6...");
                                    let ip = public_ip::addr_v6()
                                        .await
                                        .context("could not resolve public ipv6 needed for AAAA record")?;
                                    ipv6.replace(ip);
                                    ip
                                }
                            }
                        }
                        .to_string(),
                        _ => unimplemented!(),
                    };
                    if cf_record.content == ip {
                        // IP Match
                        info!(
                            name = cf_record.name,
                            id = cf_record.id,
                            content = cf_record.content,
                            "match"
                        );
                        matches.push(cf_record.clone());
                    } else {
                        // IP mismatch
                        warn!(
                            name = cf_record.name,
                            id = cf_record.id,
                            content = cf_record.content,
                            "mismatch"
                        );
                        mismatches.push(cf_record.clone());
                    }
                }
                None => {
                    // Invalid record, no match on zone and record
                    error!(zone = inv_zone, record = inv_record, "invalid");
                    invalid.push((inv_zone.clone(), inv_record.clone()));
                }
            }
        }
    }

    // Log summary
    info!(
        matches = matches.len(),
        mismatches = mismatches.len(),
        invalid = invalid.len(),
        "summary"
    );
    if !invalid.is_empty() {
        error!(amount = invalid.len(), "inventory contains invalid records")
    }
    if !mismatches.is_empty() {
        warn!(
            amount = mismatches.len(),
            "inventory contains outdated records"
        )
    }
    if invalid.is_empty() && mismatches.is_empty() {
        info!(records = matches.len(), "inventory is fully compliant")
    }
    Ok(CheckResult {
        _matches: matches,
        mismatches,
        invalid,
    })
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn commit(opts: &ConfigOpts) -> Result<()> {
    let CheckResult {
        mut mismatches,
        mut invalid,
        ..
    } = check(opts).await?;

    // Update outdated records
    if !mismatches.is_empty() {
        let fixed_record_ids = update(opts, &mismatches)
            .await
            .context("error updating mismatched records")?;
        mismatches.retain_mut(|r| !fixed_record_ids.contains(&r.id));
        if !mismatches.is_empty() {
            error!("{} outdated DNS records exist", mismatches.len());
        }
    }

    // Prune invalid records
    if !invalid.is_empty() {
        let new_inventory = prune(opts, &invalid).await?;
        invalid.retain(|(z, r)| new_inventory.data.contains(z, r));
        if !invalid.is_empty() {
            warn!("{} invalid records remain", invalid.len());
        }
    }

    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn watch(opts: &ConfigOpts) -> Result<()> {
    // Override force flag with true; watch is non-interactive
    let opts = ConfigOpts::builder()
        .merge(opts.clone())
        .inventory_commit_force(Some(true))
        .build();

    // Get watch interval
    let interval = Duration::from_millis(
        opts.watch.interval.context("no default interval")?,
    );
    debug!(interval_ms = interval.as_millis());

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
            trace!("awoken");
            if let Err(e) = commit(&opts).await {
                error!("{:?}", e);
            }
            trace!("sleeping...");
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct CheckResult {
    _matches: Vec<Record>,
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
        let force = opts.commit.force.context("no default force option")?;
        debug!(force_update = force);

        // Ask to fix records
        let fix = force || {
            prompt_yes_or_no(
                format!("Update {} mismatched records?", mismatches.len()),
                "Y/n",
            )?
            .unwrap_or(true)
        };
        if fix {
            info!("updating {} records...", mismatches.len());
            let token = opts
                .verify.token.as_ref()
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
                                trace!("resolving ipv4...");
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
                                trace!("resolving ipv6...");
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
                    info!(
                        id = cf_record.id,
                        name = cf_record.name,
                        "updated record"
                    );
                    updated_ids.insert(cf_record.id.clone());
                } else {
                    error!(
                        id = cf_record.id,
                        name = cf_record.name,
                        "unsuccessful record update"
                    )
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
        .path
        .clone()
        .unwrap_or_else(default_inventory_path);
    let mut inventory = Inventory::from_file(&inventory_path).await?;

    // Prune invalid records
    if !invalid.is_empty() {
        let force = opts.commit.force.context("no default force option")?;
        debug!(force_prune = force);

        // Ask to prune records
        let prune = force || {
            prompt_yes_or_no(
                format!("Prune {} invalid records?", invalid.len()),
                "Y/n",
            )?
            .unwrap_or(true)
        };
        // Prune
        if prune {
            let mut pruned = 0;
            info!("pruning {} invalid records...", invalid.len());
            for (zone_id, record_id) in invalid.iter() {
                let removed = inventory.data.remove(zone_id, record_id);
                if let Ok(true) = removed {
                    info!(zone = zone_id, record = record_id, "pruned record");
                    pruned += 1;
                } else {
                    error!(
                        zone = zone_id,
                        record = record_id,
                        "failed to prune record"
                    );
                }
            }
            if pruned > 0 {
                info!("updating inventory file...");
                // Best-effort attempt to post-process comments on inventory.
                let pp = InventoryPostProcessor::try_init(opts).await.ok();
                if pp.is_none() {
                    warn!("failed to initialize post-processor")
                }
                inventory.save(pp).await?;
                if invalid.len() == pruned {
                    info!(records_pruned = pruned, "inventory file healed");
                }
            }
        }
    }

    Ok(inventory)
}
