use std::path::PathBuf;

use crate::{
    cloudfare::{
        self,
        models::{Record, Zone},
    },
    config::{ConfigOpts, ConfigOptsList},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use regex::Regex;

/// List Cloudfare resources
#[derive(Debug, Args)]
#[clap(name = "list")]
pub struct List {
    #[clap(subcommand)]
    action: Option<ListSubcommands>,
    #[clap(flatten)]
    pub cfg: ConfigOptsList,
}

#[derive(Clone, Debug, Subcommand)]
enum ListSubcommands {
    /// Show zones (domains, subdomains, and identities)
    Zones,
    /// Show authoritative DNS records
    Records(RecordArgs),
}

#[derive(Debug, Clone, Args)]
pub struct RecordArgs {
    /// Print records belonging to a single zone
    #[clap(short, long)]
    pub zone: Option<String>,
}

impl List {
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
        let token = match opts
            .verify
            .as_ref()
            .map(|opts| opts.token.clone())
            .flatten()
        {
            Some(t) => t,
            None => anyhow::bail!("no token was provided"),
        };

        // Get zones
        let mut zones = cloudfare::endpoints::zones(&token).await?;
        filter_zones(&mut zones, &opts)?;

        match self.action {
            Some(subcommand) => match subcommand {
                ListSubcommands::Zones => {
                    print_zones(&zones);
                }
                ListSubcommands::Records(args) => {
                    if let Some(zone) = args.zone {
                        zones = zones
                            .into_iter()
                            .filter(|z| z.name == zone || z.id == zone)
                            .collect();
                        anyhow::ensure!(
                            zones.len() > 0,
                            "no results with that zone filter"
                        );
                    }
                    let mut records =
                        cloudfare::endpoints::records(&zones, &token).await?;
                    filter_records(&mut records, &opts)?;
                    print_records(&records);
                }
            },
            None => {
                // Get records
                let mut records =
                    cloudfare::endpoints::records(&zones, &token).await?;
                filter_records(&mut records, &opts)?;
                print_all(&zones, &records);
            }
        }

        Ok(())
    }
}

fn print_all(zones: &Vec<Zone>, records: &Vec<Record>) {
    print_zones(zones);
    print_records(records);
}

fn print_zones(zones: &Vec<Zone>) {
    println!("{:#?}", zones);
}

fn print_records(records: &Vec<Record>) {
    println!("{:#?}", records);
}

fn filter_zones(zones: &mut Vec<Zone>, opts: &ConfigOpts) -> Result<()> {
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

fn filter_records(records: &mut Vec<Record>, opts: &ConfigOpts) -> Result<()> {
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
