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
    /// Print zones matching a regex filter.
    #[clap(short, long, value_name = "name|id")]
    pub zone: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct RecordArgs {
    /// Print zones matching a regex filter.
    #[clap(short, long, value_name = "name|id")]
    pub zone: Option<String>,
    /// Print records matching a regex filter.
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

    println!("Retrieving Cloudflare resources...");
    // Get zones
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    filter_zones(&mut zones, opts)?;
    // Get records
    let mut records = cloudflare::endpoints::records(&zones, &token).await?;
    filter_records(&mut records, opts)?;

    for zone in zones.iter() {
        println!("{}", zone);
        for record in records.iter().filter(|r| r.zone_id == zone.id) {
            println!("  - {}", record);
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
    println!("Retrieving Cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    // Filter zones
    if let Some(ref zone_filter) = cmd_args.zone {
        let pattern =
            Regex::new(zone_filter).context("compiling zone regex filter")?;
        zones.retain(|z| pattern.is_match(&z.id) || pattern.is_match(&z.name));
        anyhow::ensure!(!zones.is_empty(), "no results with that zone filter");
    } else {
        filter_zones(&mut zones, opts)?;
    }

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
    println!("Retrieving Cloudflare resources...");
    let mut zones = cloudflare::endpoints::zones(&token).await?;
    if let Some(ref zone_filter) = cmd_args.zone {
        let pattern =
            Regex::new(zone_filter).context("compiling zone regex filter")?;
        zones.retain(|z| pattern.is_match(&z.id) || pattern.is_match(&z.name));
        anyhow::ensure!(!zones.is_empty(), "no results with that zone filter");
    } else {
        filter_zones(&mut zones, opts)?;
    }

    // Get records
    let mut records = cloudflare::endpoints::records(&zones, &token).await?;
    // Filter records
    if let Some(ref record_filter) = cmd_args.record {
        let pattern = Regex::new(record_filter)
            .context("compiling record regex filter")?;
        records
            .retain(|r| pattern.is_match(&r.id) || pattern.is_match(&r.name));
        anyhow::ensure!(
            !records.is_empty(),
            "no results with that record filter"
        );
    } else {
        filter_records(&mut records, opts)?;
    }

    for record in records {
        println!("{}", record);
    }

    Ok(())
}

pub fn filter_zones(zones: &mut Vec<Zone>, opts: &ConfigOpts) -> Result<()> {
    // Filter zones by configuration options
    if let Some(ref list_opts) = opts.list {
        if let Some(include_filters) = list_opts.include_zones.as_ref() {
            for filter_str in include_filters {
                let pattern = Regex::new(filter_str)
                    .context("compiling include_zones regex filter")?;
                zones.retain(|z| {
                    pattern.is_match(&z.id) || pattern.is_match(&z.name)
                });
            }
        }
        if let Some(ignore_filters) = list_opts.ignore_zones.as_ref() {
            for filter_str in ignore_filters {
                let pattern = Regex::new(filter_str)
                    .context("compiling ignore_zones regex filter")?;
                zones.retain(|z| {
                    !pattern.is_match(&z.id) && !pattern.is_match(&z.name)
                });
            }
        }
    }
    Ok(())
}

pub fn filter_records(
    records: &mut Vec<Record>,
    opts: &ConfigOpts,
) -> Result<()> {
    // Filter records by configuration options
    if let Some(ref list_opts) = opts.list {
        if let Some(include_filters) = list_opts.include_records.as_ref() {
            for filter_str in include_filters {
                let pattern = Regex::new(filter_str)
                    .context("compiling include_records regex filter")?;
                records.retain(|r| {
                    pattern.is_match(&r.id) || pattern.is_match(&r.name)
                });
            }
        }
        if let Some(ignore_filters) = list_opts.ignore_records.as_ref() {
            for filter_str in ignore_filters {
                let pattern = Regex::new(filter_str)
                    .context("compiling ignore_records regex filter")?;
                records.retain(|r| {
                    !pattern.is_match(&r.id) && !pattern.is_match(&r.name)
                });
            }
        }
    }
    Ok(())
}
