use crate::config::builder::ConfigBuilder;
use crate::config::default_config_path;
use crate::inventory::default_inventory_path;
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fmt::Debug, fmt::Display};
use tracing::debug;

/// The model of all configuration options which can be saved in a config file.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigOpts {
    pub verify: ConfigOptsVerify,
    pub list: ConfigOptsList,
    pub inventory: ConfigOptsInventory,
}

impl Default for ConfigOpts {
    /// Static default configuration options.
    fn default() -> Self {
        Self {
            verify: ConfigOptsVerify { token: None },
            list: ConfigOptsList {
                include_zones: Some(vec![".*".to_string()]),
                ignore_zones: Some(vec![]),
                include_records: Some(vec![".*".to_string()]),
                ignore_records: Some(vec![]),
            },
            inventory: ConfigOptsInventory {
                path: Some(default_inventory_path()),
                force_update: Some(false),
                force_prune: Some(false),
                watch_interval: Some(30_000),
            },
        }
    }
}

impl ConfigOpts {
    /// Return a new configuration builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Read runtime config from a target path.
    pub fn from_file(path: Option<PathBuf>) -> Result<Option<Self>> {
        let path = path.unwrap_or(default_config_path());
        if path.exists() {
            debug!("configuration file found");
            debug!("reading configuration path: '{}'", path.display());
            let cfg_bytes =
                std::fs::read_to_string(path).context("reading config file")?;
            let cfg: ConfigBuilder = toml::from_str(&cfg_bytes)
                .context("reading config file contents as TOML data")?;
            Ok(Some(cfg.build()))
        } else {
            debug!("configuration file not found");
            Ok(None)
        }
    }

    /// Read runtime config from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(ConfigOpts {
            verify: envy::prefixed("CDDNS_VERIFY_")
                .from_env::<ConfigOptsVerify>()
                .context("reading verify env var config")?,
            list: envy::prefixed("CDDNS_LIST_")
                .from_env::<ConfigOptsList>()
                .context("reading list env var config")?,
            inventory: envy::prefixed("CDDNS_INVENTORY_")
                .from_env::<ConfigOptsInventory>()
                .context("reading inventory env var config")?,
        })
    }
}

fn __display<T>(opt: Option<&T>) -> String
where
    T: Serialize + Debug,
{
    if let Some(opt) = opt {
        match ron::to_string(opt) {
            Ok(ron) => ron,
            Err(_) => format!("{opt:?}"),
        }
    } else {
        "None".to_string()
    }
}

impl Display for ConfigOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        try {
            // Verify
            writeln!(f, "Token: {}", __display(self.verify.token.as_ref()))?;

            // List
            writeln!(
                f,
                "Include zones: {}",
                __display(self.list.include_zones.as_ref())
            )?;
            writeln!(
                f,
                "Ignore zones: {}",
                __display(self.list.ignore_zones.as_ref())
            )?;
            writeln!(
                f,
                "Include records: {}",
                __display(self.list.include_records.as_ref())
            )?;
            writeln!(
                f,
                "Ignore records: {}",
                __display(self.list.ignore_records.as_ref())
            )?;

            // Inventory
            writeln!(
                f,
                "Inventory path: {}",
                __display(self.inventory.path.as_ref())
            )?;
            writeln!(
                f,
                "Force update without user prompt: {}",
                __display(self.inventory.force_update.as_ref())
            )?;
            writeln!(
                f,
                "Force prune without user prompt: {}",
                __display(self.inventory.force_prune.as_ref())
            )?;
            write!(
                f,
                "Watch interval: {}",
                __display(self.inventory.watch_interval.as_ref())
            )?;
        }
    }
}

/// Config options for the verify system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsVerify {
    // Your Cloudflare API key token.
    #[clap(short, long, env = "CDDNS_VERIFY_TOKEN", value_name = "token")]
    pub token: Option<String>,
}

/// Config options for the list system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsList {
    /// Include cloudflare zones by regex. [default: all]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_INCLUDE_ZONES"
    )]
    pub include_zones: Option<Vec<String>>,
    /// Ignore cloudflare zones by regex. [default: none]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_IGNORE_ZONES"
    )]
    pub ignore_zones: Option<Vec<String>>,

    /// Include cloudflare zone records by regex. [default: all]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_INCLUDE_RECORDS"
    )]
    pub include_records: Option<Vec<String>>,
    /// Ignore cloudflare zone records by regex. [default: none]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_IGNORE_RECORDS"
    )]
    pub ignore_records: Option<Vec<String>>,
}

/// Config options for the inventory system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsInventory {
    /// The path to the inventory file.
    #[clap(short, long, env = "CDDNS_INVENTORY_PATH", value_name = "file")]
    pub path: Option<PathBuf>,
    /// Skip prompts asking to update outdated DNS records.
    #[clap(long, env = "CDDNS_INVENTORY_FORCE_UPDATE", value_name = "boolean")]
    pub force_update: Option<bool>,
    /// Skip prompts asking to prune invalid DNS records.
    #[clap(long, env = "CDDNS_INVENTORY_FORCE_PRUNE", value_name = "boolean")]
    pub force_prune: Option<bool>,
    /// The interval for refreshing inventory records in milliseconds.
    #[clap(
        short,
        long,
        value_name = "ms",
        env = "CDDNS_INVENTORY_WATCH_INTERVAL"
    )]
    pub watch_interval: Option<u64>,
}
