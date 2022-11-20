use crate::{config::default_config_path, inventory::default_inventory_path};
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{fmt::Debug, fmt::Display};

/// A model of all potential configuration options for the CDDNS CLI system.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ConfigOpts {
    pub verify: Option<ConfigOptsVerify>,
    pub list: Option<ConfigOptsList>,
    pub inventory: Option<ConfigOptsInventory>,
    pub commit: Option<ConfigOptsCommit>,
    pub watch: Option<ConfigOptsWatch>,
}

impl ConfigOpts {
    /// Return runtime layered configuration.
    /// Configuration data precedence: Default < TOML < ENV < CLI
    pub fn full(
        toml: Option<PathBuf>,
        cli_cfg: Option<ConfigOpts>,
    ) -> Result<Self> {
        // Start with base layer (Defaults)
        let mut cfg = Self::new();
        // Apply TOML > Default
        if let Some(path) = toml.or_else(default_config_path) {
            if path.exists() {
                let toml_cfg = Self::from_file(
                    path.canonicalize().with_context(|| {
                        format!(
                            "could not canonicalize path to config file {:?}",
                            &path
                        )
                    })?,
                )?;
                cfg = cfg.merge(toml_cfg);
            }
        };
        // Apply ENV > TOML
        let env_cfg = Self::from_env()?;
        cfg = cfg.merge(env_cfg);
        // Apply CLI > ENV
        if let Some(cli_cfg) = cli_cfg {
            cfg = cfg.merge(cli_cfg);
        }
        // Return layers
        Ok(cfg)
    }

    /// New configuration, initialized to defaults.
    pub fn new() -> Self {
        Self {
            verify: Some(ConfigOptsVerify::default()),
            list: Some(ConfigOptsList::default()),
            inventory: Some(ConfigOptsInventory::default()),
            commit: Some(ConfigOptsCommit::default()),
            watch: Some(ConfigOptsWatch::default()),
        }
    }

    /// Read runtime config from a target path.
    pub fn from_file(path: PathBuf) -> Result<Self> {
        let cfg_bytes = std::fs::read(path).context("reading config file")?;
        let cfg: Self = toml::from_slice(&cfg_bytes)
            .context("reading config file contents as TOML data")?;
        Ok(cfg)
    }

    /// Read runtime config from environment variables.
    pub fn from_env() -> Result<Self> {
        Ok(ConfigOpts {
            verify: Some(
                envy::prefixed("CDDNS_VERIFY_")
                    .from_env::<ConfigOptsVerify>()
                    .context("reading verify env var config")?,
            ),
            list: Some(
                envy::prefixed("CDDNS_LIST_")
                    .from_env::<ConfigOptsList>()
                    .context("reading list env var config")?,
            ),
            inventory: Some(
                envy::prefixed("CDDNS_INVENTORY_")
                    .from_env::<ConfigOptsInventory>()
                    .context("reading inventory env var config")?,
            ),
            commit: Some(
                envy::prefixed("CDDNS_COMMIT_")
                    .from_env::<ConfigOptsCommit>()
                    .context("reading commit env var config")?,
            ),
            watch: Some(
                envy::prefixed("CDDNS_WATCH_")
                    .from_env::<ConfigOptsWatch>()
                    .context("reading watch env var config")?,
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
                    Some(g)
                }
            };
        greater.commit = match (self.commit.take(), greater.commit.take()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(l), Some(mut g)) => {
                g.force = g.force || l.force;
                Some(g)
            }
        };
        greater.watch = match (self.watch.take(), greater.watch.take()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(l), Some(mut g)) => {
                g.interval = g.interval.or(l.interval);
                Some(g)
            }
        };
        greater
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
            writeln!(
                f,
                "Token: {}",
                __display(self.verify.as_ref().and_then(|v| v.token.as_ref()))
            )?;

            // List
            writeln!(
                f,
                "Include zones: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.include_zones.as_ref())
                )
            )?;
            writeln!(
                f,
                "Ignore zones: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.ignore_zones.as_ref())
                )
            )?;
            writeln!(
                f,
                "Include records: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.include_records.as_ref())
                )
            )?;
            writeln!(
                f,
                "Ignore records: {}",
                __display(
                    self.list.as_ref().and_then(|l| l.ignore_records.as_ref())
                )
            )?;

            // Inventory
            writeln!(
                f,
                "Inventory path: {}",
                __display(
                    self.inventory.as_ref().and_then(|i| i.path.as_ref())
                )
            )?;
            writeln!(
                f,
                "Commit without user prompt (force): {}",
                __display(self.commit.as_ref().map(|c| &c.force))
            )?;
            writeln!(
                f,
                "Watch interval: {}",
                __display(
                    self.watch.as_ref().and_then(|w| w.interval.as_ref())
                )
            )?;
        };
        result
    }
}

/// Config options for the list system.
#[derive(Clone, Debug, Serialize, Deserialize, Args)]
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
/// Default configuration for list.
impl Default for ConfigOptsList {
    fn default() -> Self {
        Self {
            include_zones: Some(vec![".*".to_string()]),
            ignore_zones: None,
            include_records: Some(vec![".*".to_string()]),
            ignore_records: None,
        }
    }
}

/// Config options for the verify system.
#[derive(Clone, Debug, Serialize, Deserialize, Args)]
pub struct ConfigOptsVerify {
    // Your Cloudflare API key token
    #[clap(short, long, env = "CDDNS_VERIFY_TOKEN", value_name = "token")]
    pub token: Option<String>,
}
/// Default configuration for verify.
impl Default for ConfigOptsVerify {
    fn default() -> Self {
        Self { token: None }
    }
}

/// Config options for the inventory system.
#[derive(Clone, Debug, Serialize, Deserialize, Args)]
pub struct ConfigOptsInventory {
    /// The path to the inventory file.
    #[clap(short, long, env = "CDDNS_INVENTORY_PATH", value_name = "file")]
    pub path: Option<PathBuf>,
}
/// Default configuration for inventory.
impl Default for ConfigOptsInventory {
    fn default() -> Self {
        Self {
            path: Some(default_inventory_path()),
        }
    }
}

/// Config options for `inventory commit`.
#[derive(Clone, Debug, Serialize, Deserialize, Args)]
pub struct ConfigOptsCommit {
    /// Do not prompt, forcibly commit.
    #[clap(short, long, env = "CDDNS_COMMIT_FORCE")]
    #[serde(default)]
    pub force: bool,
}
/// Default configuration for inventory commit.
impl Default for ConfigOptsCommit {
    fn default() -> Self {
        Self { force: false }
    }
}

/// Config options for `inventory watch`.
#[derive(Clone, Debug, Serialize, Deserialize, Args)]
pub struct ConfigOptsWatch {
    /// The interval for refreshing inventory records in milliseconds.
    #[clap(short, long, value_name = "ms", env = "CDDNS_WATCH_INTERVAL")]
    pub interval: Option<u64>,
}
/// Default configuration for inventory watch.
impl Default for ConfigOptsWatch {
    fn default() -> Self {
        Self {
            interval: Some(5000),
        }
    }
}
