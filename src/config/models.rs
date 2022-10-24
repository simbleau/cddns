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
    /// Return the full configuration based on config file & environment
    /// variables.
    pub fn full(config: Option<PathBuf>) -> Result<Self> {
        let toml_cfg = Self::from_file(config)?;
        let env_cfg =
            Self::from_env().context("error reading env var config")?;
        let cfg = Self::merge(toml_cfg, env_cfg);
        Ok(cfg)
    }

    /// Read runtime config from a target path.
    fn from_file(path: Option<PathBuf>) -> Result<Self> {
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
    fn from_env() -> Result<Self> {
        Ok(ConfigOpts {
            check: Some(
                envy::prefixed("CFDDNS_CHECK_")
                    .from_env::<ConfigOptsCheck>()?,
            ),
        })
    }

    /// Merge the given layers, where the `greater` layer takes precedence.
    fn merge(mut lesser: Self, mut greater: Self) -> Self {
        greater.check = match (lesser.check.take(), greater.check.take()) {
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
    /// A filter to apply to cloudfare targets
    pub filter: Option<String>,
    /// Include cloudfare targets (default: ALL)
    #[clap(short, long, value_name = "path")]
    pub include: Option<Vec<PathBuf>>,
    /// Ignore cloudfare targets (default: NONE)
    #[clap(short, long, value_name = "path")]
    pub ignore: Option<Vec<PathBuf>>,
}
