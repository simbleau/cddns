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
        let token = match opts.verify.map(|opts| opts.token).flatten() {
            Some(t) => t,
            None => anyhow::bail!("no token was provided"),
        };

        // Get zones
        let mut zones = cloudfare::endpoints::zones(&token).await?;
        // Filter zones by configuration options
        if let Some(ref list_opts) = opts.list {
            if let Some(include) = &list_opts.include_zones {
                for filter_str in include {
                    zones = include_zones(zones, filter_str)?;
                }
            }
            if let Some(ignore) = &list_opts.ignore_zones {
                for filter_str in ignore {
                    zones = ignore_zones(zones, filter_str)?;
                }
            }
        }

        match self.action {
            Some(subcommand) => match subcommand {
                ListSubcommands::Zones => {
                    print_zones(&zones);
                }
                ListSubcommands::Records(args) => {
                    if let Some(zone) = args.zone {
                        zones =
                            filter(zones, |z| z.name == zone || z.id == zone);
                        anyhow::ensure!(
                            zones.len() > 0,
                            "no results with that zone filter"
                        );
                    }
                    let mut records =
                        cloudfare::endpoints::records(&zones, &token).await?;
                    // Filter records by configuration options
                    if let Some(list_opts) = &opts.list {
                        if let Some(include) = &list_opts.include_records {
                            for filter_str in include {
                                records = include_records(records, filter_str)?;
                            }
                        }
                        if let Some(ignore) = &list_opts.ignore_records {
                            for filter_str in ignore {
                                records = ignore_records(records, filter_str)?;
                            }
                        }
                    }
                    print_records(&records);
                }
            },
            None => {
                // Get records
                let mut records =
                    cloudfare::endpoints::records(&zones, &token).await?;
                // Filter records by configuration options
                if let Some(list_opts) = &opts.list {
                    if let Some(include) = &list_opts.include_records {
                        for filter_str in include {
                            records = include_records(records, filter_str)?;
                        }
                    }
                    if let Some(ignore) = &list_opts.ignore_records {
                        for filter_str in ignore {
                            records = ignore_records(records, filter_str)?;
                        }
                    }
                }
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

fn include_zones(items: Vec<Zone>, filter_str: &str) -> Result<Vec<Zone>> {
    let pattern = Regex::new(filter_str)
        .context("error compiling include_zones regex filter")?;
    Ok(filter(items, |r| {
        pattern.is_match(&r.id) || pattern.is_match(&r.name)
    }))
}

fn ignore_zones(items: Vec<Zone>, filter_str: &str) -> Result<Vec<Zone>> {
    let pattern = Regex::new(filter_str)
        .context("error compiling ignore_zones regex filter")?;
    Ok(filter(items, |r| {
        !pattern.is_match(&r.id) && !pattern.is_match(&r.name)
    }))
}

fn include_records(
    items: Vec<Record>,
    filter_str: &str,
) -> Result<Vec<Record>> {
    let pattern = Regex::new(filter_str)
        .context("error compiling include_records regex filter")?;
    Ok(filter(items, |r| {
        pattern.is_match(&r.id) || pattern.is_match(&r.name)
    }))
}

fn ignore_records(items: Vec<Record>, filter_str: &str) -> Result<Vec<Record>> {
    let pattern = Regex::new(filter_str)
        .context("error compiling ignore_records regex filter")?;
    Ok(filter(items, |r| {
        !pattern.is_match(&r.id) && !pattern.is_match(&r.name)
    }))
}

fn filter<T, P>(items: Vec<T>, predicate: P) -> Vec<T>
where
    T: Sized,
    P: FnMut(&T) -> bool,
{
    items.into_iter().filter(predicate).collect()
}
