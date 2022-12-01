use crate::{config::default_config_path, inventory::default_inventory_path};
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::{fmt::Debug, fmt::Display};
use tracing::debug;

use super::builder::ConfigBuilder;

/// A model of all configuration options for the CDDNS system.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigOpts {
    pub verify: ConfigOptsVerify,
    pub list: ConfigOptsList,
    pub inventory: ConfigOptsInventory,
    pub commit: ConfigOptsInventoryCommit,
    pub watch: ConfigOptsInventoryWatch,
}

impl Default for ConfigOpts {
    /// Static default configuration options.
    fn default() -> Self {
        Self {
            verify: ConfigOptsVerify { token: None },
            list: ConfigOptsList {
                include_zones: Some(vec![".*".to_string()]),
                ignore_zones: None,
                include_records: Some(vec![".*".to_string()]),
                ignore_records: None,
            },
            inventory: ConfigOptsInventory {
                path: Some(default_inventory_path()),
            },
            commit: ConfigOptsInventoryCommit { force: Some(false) },
            watch: ConfigOptsInventoryWatch {
                interval: Some(30_000),
            },
        }
    }
}

impl ConfigOpts {
    /// Return a new configuration builder.
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::new()
    }

    /// Return runtime layered configuration.
    /// Configuration data precedence: Default < TOML < ENV < CLI
    pub fn full(
        toml: Option<impl AsRef<Path>>,
        cli_cfg: Option<ConfigOpts>,
    ) -> Result<Self> {
        let mut layers = vec![];
        // CLI > ENV
        if let Some(cli_cfg) = cli_cfg {
            layers.push(cli_cfg);
        }
        // ENV > TOML
        let env_cfg = Self::from_env()?;
        layers.push(env_cfg);
        // TOML > Default
        let toml: Option<PathBuf> = toml.map(|p| p.as_ref().to_owned());
        if let Some(path) = toml.or_else(default_config_path) {
            if path.exists() {
                debug!("configuration file found");
                let toml_cfg = Self::from_file(
                    path.canonicalize().with_context(|| {
                        format!(
                            "could not canonicalize path to config file {:?}",
                            &path
                        )
                    })?,
                )?;
                layers.push(toml_cfg);
            } else {
                debug!("configuration file not found");
            }
        } else {
            debug!("no default configuration file path");
        };
        // Default as lowest priority
        layers.push(Self::default());
        // Apply layering
        let mut cfg_builder = Self::builder();
        while let Some(cfg) = layers.pop() {
            cfg_builder = cfg_builder.merge(cfg);
        }
        Ok(cfg_builder.build())
    }

    /// Read runtime config from a target path.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        debug!("reading configuration path: {}", path.display());
        let cfg_bytes = std::fs::read(path).context("reading config file")?;
        let cfg: ConfigBuilder = toml::from_slice(&cfg_bytes)
            .context("reading config file contents as TOML data")?;
        Ok(cfg.build())
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
            commit: envy::prefixed("CDDNS_COMMIT_")
                .from_env::<ConfigOptsInventoryCommit>()
                .context("reading commit env var config")?,
            watch: envy::prefixed("CDDNS_WATCH_")
                .from_env::<ConfigOptsInventoryWatch>()
                .context("reading watch env var config")?,
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
            Err(_) => format!("{:?}", opt),
        }
    } else {
        "None".to_string()
    }
}

impl Display for ConfigOpts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = try {
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
                "Commit without user prompt (force): {}",
                __display(Some(&self.commit.force))
            )?;
            write!(
                f,
                "Watch interval: {}",
                __display(self.watch.interval.as_ref())
            )?;
        };
        result
    }
}

/// Config options for the verify system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsVerify {
    // Your Cloudflare API key token
    #[clap(short, long, env = "CDDNS_VERIFY_TOKEN", value_name = "token")]
    pub token: Option<String>,
}

/// Config options for the list system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsList {
    /// Include cloudflare zones by regex [default: all]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_INCLUDE_ZONES"
    )]
    pub include_zones: Option<Vec<String>>,
    /// Ignore cloudflare zones by regex [default: none]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_IGNORE_ZONES"
    )]
    pub ignore_zones: Option<Vec<String>>,

    /// Include cloudflare zone records by regex [default: all]
    #[clap(
        long,
        value_name = "pattern1,pattern2,..",
        env = "CDDNS_LIST_INCLUDE_RECORDS"
    )]
    pub include_records: Option<Vec<String>>,
    /// Ignore cloudflare zone records by regex [default: none]
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
}

/// Config options for `inventory commit`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsInventoryCommit {
    /// Do not prompt, forcibly commit.
    #[clap(short, long, env = "CDDNS_COMMIT_FORCE")]
    pub force: Option<bool>,
}

/// Config options for `inventory watch`.
#[derive(Clone, Debug, Default, Serialize, Deserialize, Args)]
pub struct ConfigOptsInventoryWatch {
    /// The interval for refreshing inventory records in milliseconds.
    #[clap(short, long, value_name = "ms", env = "CDDNS_WATCH_INTERVAL")]
    pub interval: Option<u64>,
}
