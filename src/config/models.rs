use crate::config::CONFIG_PATH;
use anyhow::{Context, Result};
use clap::Args;
use serde::Deserialize;
use std::path::PathBuf;

/// A model of all potential configuration options for the CFDDNS CLI system.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConfigOpts {
    pub verify: Option<ConfigOptsVerify>,
    pub list: Option<ConfigOptsList>,
}

impl ConfigOpts {
    /// Read runtime config from a target path.
    pub fn from_file(path: Option<PathBuf>) -> Result<Self> {
        let mut cfddns_toml_path = path.unwrap_or_else(|| CONFIG_PATH.into());
        if !cfddns_toml_path.exists() {
            return Ok(Default::default());
        }
        if !cfddns_toml_path.is_absolute() {
            cfddns_toml_path =
                cfddns_toml_path.canonicalize().with_context(|| {
                    format!(
                    "error getting canonical path to CFDDNS config file {:?}",
                    &cfddns_toml_path
                )
                })?;
        }
        let cfg_bytes = std::fs::read(&cfddns_toml_path)
            .context("error reading config file")?;
        let cfg: Self = toml::from_slice(&cfg_bytes)
            .context("error reading config file contents as TOML data")?;
        Ok(cfg)
    }

    /// Read runtime config from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(ConfigOpts {
            list: Some(
                envy::prefixed("CFDDNS_LIST_")
                    .from_env::<ConfigOptsList>()
                    .context("error reading env var config")?,
            ),
            verify: Some(
                envy::prefixed("CFDDNS_VERIFY_")
                    .from_env::<ConfigOptsVerify>()
                    .context("error reading env var config")?,
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
        greater
    }
}

/// Config options for the list system.
#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct ConfigOptsList {
    /// Include cloudfare zones (default: ["*"])
    #[clap(long)]
    pub include_zones: Option<Vec<String>>,
    /// Ignore cloudfare zones (default: [])
    #[clap(long)]
    pub ignore_zones: Option<Vec<String>>,

    /// Include cloudfare records (default: ["*"])
    #[clap(long)]
    pub include_records: Option<Vec<String>>,
    /// Ignore cloudfare records (default: [])
    #[clap(long)]
    pub ignore_records: Option<Vec<String>>,
}

/// Config options for the verify system.
#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct ConfigOptsVerify {
    // Your Cloudfare API key token
    #[clap(short, long)]
    pub token: Option<String>,
}
