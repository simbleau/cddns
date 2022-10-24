use crate::config::CONFIG_PATH;
use anyhow::{Context, Result};
use clap::Args;
use serde::Deserialize;
use std::path::PathBuf;

/// A model of all potential configuration options for the CFDDNS CLI system.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConfigOpts {
    pub check: Option<ConfigOptsCheck>,
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
            check: Some(
                envy::prefixed("CFDDNS_CHECK_")
                    .from_env::<ConfigOptsCheck>()
                    .context("error reading env var config")?,
            ),
        })
    }

    /// Merge config layers, where the `greater` layer takes precedence.
    pub fn merge(mut self, mut greater: Self) -> Self {
        greater.check = match (self.check.take(), greater.check.take()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(_), Some(g)) => Some(g),
        };
        greater
    }
}

/// Config options for the check system.
#[derive(Clone, Debug, Default, Deserialize, Args)]
pub struct ConfigOptsCheck {
    // A zone to check
    pub zone: Option<String>,
    /// A filter to apply to cloudfare targets
    pub filter: Option<String>,
    /// Ignore cloudfare targets (default: NONE)
    #[clap(short, long, value_name = "path")]
    pub ignore: Option<Vec<String>>,
}
