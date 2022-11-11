use crate::config::DEFAULT_CONFIG_PATH;
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A model of all potential configuration options for the CFDDNS CLI system.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigOpts {
    pub verify: Option<ConfigOptsVerify>,
    pub list: Option<ConfigOptsList>,
    pub inventory: Option<ConfigOptsInventory>,
}

impl ConfigOpts {
    /// Read runtime config from a target path.
    pub fn from_file(path: Option<PathBuf>) -> Result<Self> {
        let mut cfddns_toml_path = path.unwrap_or(DEFAULT_CONFIG_PATH.into());
        if !cfddns_toml_path.exists() {
            return Ok(Default::default());
        }
        if !cfddns_toml_path.is_absolute() {
            cfddns_toml_path =
                cfddns_toml_path.canonicalize().with_context(|| {
                    format!(
                    "could not canonicalize path to CFDDNS config file {:?}",
                    &cfddns_toml_path
                )
                })?;
        }
        let cfg_bytes =
            std::fs::read(&cfddns_toml_path).context("reading config file")?;
        let cfg: Self = toml::from_slice(&cfg_bytes)
            .context("reading config file contents as TOML data")?;
        Ok(cfg)
    }

    /// Read runtime config from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(ConfigOpts {
            verify: Some(
                envy::prefixed("CFDDNS_VERIFY_")
                    .from_env::<ConfigOptsVerify>()
                    .context("reading verify env var config")?,
            ),
            list: Some(
                envy::prefixed("CFDDNS_LIST_")
                    .from_env::<ConfigOptsList>()
                    .context("reading list env var config")?,
            ),
            inventory: Some(
                envy::prefixed("CFDDNS_INVENTORY_")
                    .from_env::<ConfigOptsInventory>()
                    .context("reading inventory env var config")?,
            ),
        })
    }

    /// Merge config layers, where the `greater` layer takes precedence.
    pub fn merge(mut self, mut greater: Self) -> Self {
        greater.verify = match (self.verify.take(), greater.verify.take()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(l), Some(mut g)) => {
                g.token = g.token.or(l.token);
                Some(g)
            }
        };
        greater.list = match (self.list.take(), greater.list.take()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(l), Some(mut g)) => {
                g.include_zones = g.include_zones.or(l.include_zones);
                g.ignore_zones = g.ignore_zones.or(l.ignore_zones);
                g.include_records = g.include_records.or(l.include_records);
                g.ignore_records = g.ignore_records.or(l.ignore_records);
                Some(g)
            }
        };
        greater.inventory =
            match (self.inventory.take(), greater.inventory.take()) {
                (None, None) => None,
                (Some(val), None) | (None, Some(val)) => Some(val),
                (Some(l), Some(mut g)) => {
                    g.path = g.path.or(l.path);
                    g.interval = g.interval.or(l.interval);
                    Some(g)
                }
            };
        greater
    }
}

/// Config options for the list system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsList {
    /// Include cloudflare zones by regex [default: all]
    #[clap(long, value_name = "pattern")]
    pub include_zones: Option<Vec<String>>,
    /// Ignore cloudflare zones by regex [default: none]
    #[clap(long, value_name = "pattern")]
    pub ignore_zones: Option<Vec<String>>,

    /// Include cloudflare zone records by regex [default: all]
    #[clap(long, value_name = "pattern")]
    pub include_records: Option<Vec<String>>,
    /// Ignore cloudflare zone records by regex [default: none]
    #[clap(long, value_name = "pattern")]
    pub ignore_records: Option<Vec<String>>,
}

/// Config options for the verify system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsVerify {
    // Your Cloudflare API key token
    #[clap(short, long, env = "CFDDNS_TOKEN", value_name = "token")]
    pub token: Option<String>,
}

/// Config options for the inventory system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsInventory {
    /// The path to the inventory file.
    #[clap(short, long, env = "CFDDNS_INVENTORY", value_name = "file")]
    pub path: Option<PathBuf>,
    /// The interval for watching inventory records.
    #[clap(short, long, value_name = "milliseconds")]
    pub interval: Option<u64>,
}
