use std::path::PathBuf;

use crate::{
    cloudfare::{
        self,
        models::{Record, Zone},
    },
    config::models::{ConfigOpts, ConfigOptsList},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use regex::Regex;

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
    /// Show zones (domains, subdomains, and identities)
    Zones(ZoneArgs),
    /// Show authoritative DNS records
    Records(RecordArgs),
}

#[derive(Debug, Clone, Args)]
pub struct ZoneArgs {
    /// Print a single zone
    #[clap(short, long, value_name = "name|id")]
    pub zone: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct RecordArgs {
    /// Print records belonging to a single zone
    #[clap(short, long, value_name = "name|id")]
    pub zone: Option<String>,
    /// Print a single record
    #[clap(short, long, value_name = "name|id")]
    pub record: Option<String>,
}

impl ListCmd {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            list: Some(self.cfg),
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

        match self.action {
            Some(subcommand) => match subcommand {
                ListSubcommands::Zones(args) => {
                    // Get zones
                    let mut zones = cloudfare::endpoints::zones(&token).await?;
                    // Filter zones
                    if let Some(zone) = args.zone {
                        zones.retain(|z| z.name == zone || z.id == zone);
                        anyhow::ensure!(
                            zones.len() > 0,
                            "no results with that zone filter"
                        );
                    } else {
                        filter_zones(&mut zones, &opts)?;
                    }
                    print_zones(&zones);
                }
                ListSubcommands::Records(args) => {
                    // Get zones
                    let mut zones = cloudfare::endpoints::zones(&token).await?;
                    if let Some(zone) = args.zone {
                        zones = zones
                            .into_iter()
                            .filter(|z| z.name == zone || z.id == zone)
                            .collect();
                        anyhow::ensure!(
                            zones.len() > 0,
                            "no results with that zone filter"
                        );
                    } else {
                        filter_zones(&mut zones, &opts)?;
                    }
                    // Get records
                    let mut records =
                        cloudfare::endpoints::records(&zones, &token).await?;
                    // Filter records
                    if let Some(record) = args.record {
                        records = records
                            .into_iter()
                            .filter(|r| r.name == record || r.id == record)
                            .collect();
                        anyhow::ensure!(
                            records.len() > 0,
                            "no results with that record filter"
                        );
                    } else {
                        filter_records(&mut records, &opts)?;
                    }
                    print_records(&records);
                }
            },
            None => {
                // Get zones
                let mut zones = cloudfare::endpoints::zones(&token).await?;
                // Get records
                let mut records =
                    cloudfare::endpoints::records(&zones, &token).await?;
                filter_zones(&mut zones, &opts)?;
                filter_records(&mut records, &opts)?;
                print_all(&zones, &records);
            }
        }

        Ok(())
    }
}

fn print_all(zones: &Vec<Zone>, records: &Vec<Record>) {
    for (i, zone) in zones.iter().enumerate() {
        if i != 0 {
            println!("{}", "-".repeat(3));
        }
        println!("{}", zone);
        for record in records.iter().filter(|r| r.zone_id == zone.id) {
            println!("  - {}", record);
        }
    }
}

fn print_zones(zones: &Vec<Zone>) {
    for zone in zones {
        println!("{}", zone);
    }
}

fn print_records(records: &Vec<Record>) {
    for record in records {
        println!("{}", record);
    }
}

pub fn filter_zones(zones: &mut Vec<Zone>, opts: &ConfigOpts) -> Result<()> {
    // Filter zones by configuration options
    if let Some(ref list_opts) = opts.list {
        if let Some(include_filters) = list_opts.include_zones.as_ref() {
            for filter_str in include_filters {
                let pattern = Regex::new(&filter_str)
                    .context("error compiling include_zones regex filter")?;
                zones.retain(|z| {
                    pattern.is_match(&z.id) || pattern.is_match(&z.name)
                });
            }
        }
        if let Some(ignore_filters) = list_opts.ignore_zones.as_ref() {
            for filter_str in ignore_filters {
                let pattern = Regex::new(&filter_str)
                    .context("error compiling ignore_zones regex filter")?;
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
                let pattern = Regex::new(&filter_str)
                    .context("error compiling include_records regex filter")?;
                records.retain(|r| {
                    pattern.is_match(&r.id) || pattern.is_match(&r.name)
                });
            }
        }
        if let Some(ignore_filters) = list_opts.ignore_records.as_ref() {
            for filter_str in ignore_filters {
                let pattern = Regex::new(&filter_str)
                    .context("error compiling ignore_records regex filter")?;
                records.retain(|r| {
                    !pattern.is_match(&r.id) && !pattern.is_match(&r.name)
                });
            }
        }
    }
    Ok(())
}
