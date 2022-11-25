use crate::{
    cloudflare::{
        self,
        models::{Record, Zone},
    },
    config::models::{ConfigOpts, ConfigOptsList},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use regex::Regex;
use std::path::PathBuf;
use tracing::{debug, info};

/// List available resources
#[derive(Debug, Args)]
#[clap(name = "list")]
pub struct ListCmd {
    #[clap(subcommand)]
    action: Option<ListSubcommands>,
    #[clap(flatten)]
    pub cfg: ConfigOptsList,
}

#[derive(Clone, Debug, Subcommand)]
enum ListSubcommands {
    /// Show zones (domains, subdomains, and identities.)
    Zones(ZoneArgs),
    /// Show authoritative DNS records.
    Records(RecordArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ZoneArgs {
    /// Print a single zone matching a name or id.
    #[clap(short, long, value_name = "name|id")]
    pub zone: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct RecordArgs {
    /// Print records from a single zone matching a name or id.
    #[clap(short, long, value_name = "name|id")]
    pub zone: Option<String>,
    /// Print a single record matching a name or id.
    #[clap(short, long, value_name = "name|id")]
    pub record: Option<String>,
}

impl ListCmd {
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let cli_cfg = ConfigOpts {
            list: Some(self.cfg),
            ..Default::default()
        };
        let opts = ConfigOpts::full(config, Some(cli_cfg))?;

        match self.action {
            Some(subcommand) => match subcommand {
                ListSubcommands::Zones(args) => print_zones(&opts, &args).await,
                ListSubcommands::Records(args) => {
                    print_records(&opts, &args).await
                }
            },
            None => print_all(&opts).await,
        }
    }
}

/// Print all zones and records.
#[tracing::instrument(level = "trace", skip(opts))]
async fn print_all(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get zones
    info!("retrieving cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    retain_zones(&mut zones, opts)?;
    // Get records
    let mut records = cloudflare::endpoints::records(&zones, &token).await?;
    retain_records(&mut records, opts)?;
    debug!(
        "received {} zones with {} records",
        zones.len(),
        records.len()
    );

    // Print all
    for zone in zones.iter() {
        println!("{}", zone);
        for record in records.iter().filter(|r| r.zone_id == zone.id) {
            println!("  ↳ {}", record);
        }
    }
    Ok(())
}

/// Print only zones.
#[tracing::instrument(level = "trace", skip(opts))]
async fn print_zones(opts: &ConfigOpts, cmd_args: &ZoneArgs) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get zones
    info!("retrieving cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    // Apply filtering
    if let Some(ref zone_id) = cmd_args.zone {
        zones = vec![find_zone(&mut zones, zone_id)?
            .context("no result with that zone id/name")?];
    } else {
        retain_zones(&mut zones, opts)?;
    }

    // Print zones
    for zone in zones {
        println!("{}", zone);
    }
    Ok(())
}

/// Print only records.
#[tracing::instrument(level = "trace", skip(opts))]
async fn print_records(opts: &ConfigOpts, cmd_args: &RecordArgs) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    // Get zones
    info!("retrieving cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    if let Some(ref zone_id) = cmd_args.zone {
        zones = vec![find_zone(&mut zones, zone_id)?
            .context("no result with that zone id/name")?];
    } else {
        retain_zones(&mut zones, opts)?;
    }

    // Get records
    let mut records = cloudflare::endpoints::records(&zones, &token).await?;
    // Apply filtering
    if let Some(ref record_id) = cmd_args.record {
        records = vec![find_record(&mut records, record_id)?
            .context("no result with that record id/name")?];
    } else {
        retain_records(&mut records, opts)?;
    }

    // Print records
    for record in records {
        println!("{}", record);
    }
    Ok(())
}

/// Find a zone matching the given identifier.
pub fn find_zone(
    zones: &mut Vec<Zone>,
    id: impl Into<String>,
) -> Result<Option<Zone>> {
    let id_str = id.into();
    for z in zones {
        if id_str == z.id || id_str == z.name {
            return Ok(Some(z.clone()));
        }
    }
    Ok(None)
}

/// Retain zones matching the given configuration filters.
pub fn retain_zones(zones: &mut Vec<Zone>, opts: &ConfigOpts) -> Result<()> {
    let beginning_amt = zones.len();
    // Filter zones by configuration options
    if let Some(ref list_opts) = opts.list {
        if let Some(include_filters) = list_opts.include_zones.as_ref() {
            for filter_str in include_filters {
                debug!("applying include filter: '{}'", filter_str);
                let pattern = Regex::new(filter_str)
                    .context("compiling include_zones regex filter")?;
                zones.retain(|z| {
                    pattern.is_match(&z.id) || pattern.is_match(&z.name)
                });
            }
        }
        if let Some(ignore_filters) = list_opts.ignore_zones.as_ref() {
            for filter_str in ignore_filters {
                debug!("applying ignore filter: '{}'", filter_str);
                let pattern = Regex::new(filter_str)
                    .context("compiling ignore_zones regex filter")?;
                zones.retain(|z| {
                    !pattern.is_match(&z.id) && !pattern.is_match(&z.name)
                });
            }
        }
    }
    debug!("filtered out {} zones", beginning_amt - zones.len());

    Ok(())
}

/// Find a record matching the given identifier.
pub fn find_record(
    records: &mut Vec<Record>,
    id: impl Into<String>,
) -> Result<Option<Record>> {
    let id_str = id.into();
    for r in records {
        if id_str == r.id || id_str == r.name {
            return Ok(Some(r.clone()));
        }
    }
    Ok(None)
}

/// Retain records matching the given configuration filters.
pub fn retain_records(
    records: &mut Vec<Record>,
    opts: &ConfigOpts,
) -> Result<()> {
    let beginning_amt = records.len();
    // Filter records by configuration options
    if let Some(ref list_opts) = opts.list {
        if let Some(include_filters) = list_opts.include_records.as_ref() {
            for filter_str in include_filters {
                debug!("applying include filter: '{}'", filter_str);
                let pattern = Regex::new(filter_str)
                    .context("compiling include_records regex filter")?;
                records.retain(|r| {
                    pattern.is_match(&r.id) || pattern.is_match(&r.name)
                });
            }
        }
        if let Some(ignore_filters) = list_opts.ignore_records.as_ref() {
            for filter_str in ignore_filters {
                debug!("applying ignore filter: '{}'", filter_str);
                let pattern = Regex::new(filter_str)
                    .context("compiling ignore_records regex filter")?;
                records.retain(|r| {
                    !pattern.is_match(&r.id) && !pattern.is_match(&r.name)
                });
            }
        }
    }
    debug!("filtered out {} records", beginning_amt - records.len());

    Ok(())
}
