use crate::{
    cloudflare::{self, endpoints::update_record, models::Record},
    config::models::{
        ConfigOpts, ConfigOptsCommit, ConfigOptsInventory, ConfigOptsWatch,
    },
    inventory::{default_inventory_path, models::Inventory},
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
                    commit: Some(ConfigOptsCommit { force: true }),
                    ..Default::default()
                };
                watch(&opts.merge(cli_cfg)).await
            }
        }
    }
}

#[tracing::instrument(level = "trace", skip(opts))]
async fn build(opts: &ConfigOpts) -> Result<()> {
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
    ensure!(!records.is_empty(), "no records to build inventory from");

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
            if inventory.contains(&zone_id, &record_id) {
                println!("✅ You already added '{}'.", selected_record.name)
            } else {
                inventory.insert(zone_id, record_id);
                println!("✅ Added '{}'.", selected_record.name);
            }
        } else {
            println!("❌ No records for this zone.")
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
    io::fs::remove_interactive(&path, &mut scanner).await?;

    // Best-effort attempt to post-process comments on inventory.
    let post_processor = InventoryPostProcessor::from(&zones, &records);
    if inventory.save(&path, Some(post_processor)).await.is_err() {
        warn!("post-processing failed for inventory file");
        inventory
            .save::<InventoryPostProcessor>(&path, None)
            .await?
    };

    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
async fn show(opts: &ConfigOpts) -> Result<()> {
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    if inventory.is_empty() {
        warn!("inventory is empty");
    } else {
        println!("{}", inventory);
    }
    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
async fn check(opts: &ConfigOpts) -> Result<CheckResult> {
    // Get inventory
    info!("reading inventory...");
    let inventory_path = opts
        .inventory
        .as_ref()
        .and_then(|opts| opts.path.clone())
        .unwrap_or_else(default_inventory_path);
    let inventory = Inventory::from_file(inventory_path).await?;

    // End early if inventory is empty
    if inventory.is_empty() {
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
    for (ref inv_zone, ref inv_records) in inventory.into_iter() {
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
        ipv4,
        ipv6,
    })
}

#[tracing::instrument(level = "trace", skip(opts))]
async fn commit(opts: &ConfigOpts) -> Result<()> {
    let CheckResult {
        matches,
        mut mismatches,
        mut invalid,
        ipv4,
        ipv6,
    } = check(opts).await?;

    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;

    let force = opts
        .commit
        .as_ref()
        .map(|opts| opts.force)
        .unwrap_or(ConfigOptsCommit::default().force);
    debug!("force flag: {}", force);

    let mut scanner = Scanner::new(tokio::runtime::Handle::current());

    // Fix mismatched records
    if !mismatches.is_empty() {
        // Ask to fix records
        let fix = force
            || scanner
                .prompt_yes_or_no(
                    format!("Fix {} mismatched records?", mismatches.len()),
                    "Y/n",
                )
                .await?
                .unwrap_or(true);
        // Track fixed records
        let mut fixed = HashSet::new();
        if fix {
            info!("fixing mismatched records...");
            for cf_record in mismatches.iter() {
                if match cf_record.record_type.as_str() {
                    "A" => {
                        update_record(
                            &token,
                            &cf_record.zone_id,
                            &cf_record.id,
                            ipv4.with_context(|| {
                                format!("need ipv4 to patch '{}'", cf_record.id)
                            })?,
                        )
                        .await
                    }
                    "AAAA" => {
                        update_record(
                            &token,
                            &cf_record.zone_id,
                            &cf_record.id,
                            ipv6.with_context(|| {
                                format!("need ipv6 to patch '{}'", cf_record.id)
                            })?,
                        )
                        .await
                    }
                    _ => unimplemented!(),
                }
                .is_ok()
                {
                    debug!("updated '{}'", &cf_record.id);
                    fixed.insert(cf_record.id.clone());
                } else {
                    error!("unsuccessful update of '{}'", &cf_record.id)
                }
            }
        }
        mismatches.retain_mut(|r| !fixed.contains(&r.id));
    }

    // Prune invalid records
    if !invalid.is_empty() {
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
            let inventory_path = opts
                .inventory
                .as_ref()
                .and_then(|opts| opts.path.clone())
                .unwrap_or_else(default_inventory_path);
            let mut inventory = Inventory::from_file(&inventory_path).await?;
            info!("removing invalid records...");
            for (zone_id, record_id) in invalid.iter() {
                let removed = inventory.remove(zone_id, record_id);
                if let Ok(true) = removed {
                    debug!("pruned '{} | {}'", &zone_id, &record_id);
                    pruned.insert((zone_id.clone(), record_id.clone()));
                } else {
                    error!("could not prune '{} | {}'", &zone_id, &record_id);
                }
            }
            invalid.retain_mut(|(z, r)| {
                !pruned.contains(&(z.to_owned(), r.to_owned()))
            });
            // Best-effort attempt to post-process comments on inventory.
            info!("saving inventory file...");
            let result: Result<()> = try {
                debug!("retrieving post-processing resources...");
                let token = opts
                    .verify
                    .as_ref()
                    .and_then(|opts| opts.token.clone())
                    .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;
                let zones =
                    cloudflare::endpoints::zones(token.to_string()).await?;
                let records =
                    cloudflare::endpoints::records(&zones, token.to_string())
                        .await?;
                info!("post-processing inventory...");
                let post_processor =
                    InventoryPostProcessor::from(&zones, &records);
                inventory
                    .save(&inventory_path, Some(post_processor))
                    .await?;
            };
            if result.is_err() {
                // Save, without post-processing
                inventory
                    .save::<InventoryPostProcessor>(&inventory_path, None)
                    .await?
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
        warn!("mismatching or invalid records remain");
    }

    Ok(())
}

#[tracing::instrument(level = "trace", skip(opts))]
pub async fn watch(opts: &ConfigOpts) -> Result<()> {
    // Override force flag with true; watch is non-interactive
    ensure!(
        opts.commit.as_ref().is_some_and(|commit| commit.force),
        "force should be true, watch is non-interactive"
    );

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
            if let Err(e) = commit(opts).await {
                error!("{:?}", e);
            }
        }
    } else {
        let mut timer = time::interval(interval);
        timer.set_missed_tick_behavior(MissedTickBehavior::Skip);
        loop {
            timer.tick().await;
            debug!("awoken");
            if let Err(e) = commit(opts).await {
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
    ipv4: Option<Ipv4Addr>,
    ipv6: Option<Ipv6Addr>,
}
