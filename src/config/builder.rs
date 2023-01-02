use crate::config::models::{
    ConfigOpts, ConfigOptsInventory, ConfigOptsList, ConfigOptsVerify,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// A builder for configuration options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigBuilder {
    pub verify: Option<ConfigOptsVerify>,
    pub list: Option<ConfigOptsList>,
    pub inventory: Option<ConfigOptsInventory>,
}

impl ConfigBuilder {
    /// Create a new config opts builder.
    pub(crate) fn new() -> Self {
        Self {
            verify: None,
            list: None,
            inventory: None,
        }
    }

    /// Merge config layers, where the `greater` layer takes precedence.
    pub fn merge(&mut self, greater: impl Into<Self>) -> &mut Self {
        let mut greater = greater.into();
        self.verify = match (self.verify.take(), greater.verify.take()) {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(l), Some(mut g)) => {
                g.token = g.token.or(l.token);
                Some(g)
            }
        };
        self.list = match (self.list.take(), greater.list.take()) {
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
        self.inventory = match (self.inventory.take(), greater.inventory.take())
        {
            (None, None) => None,
            (Some(val), None) | (None, Some(val)) => Some(val),
            (Some(l), Some(mut g)) => {
                g.path = g.path.or(l.path);
                g.force_update = g.force_update.or(l.force_update);
                g.force_prune = g.force_prune.or(l.force_prune);
                g.watch_interval = g.watch_interval.or(l.watch_interval);
                Some(g)
            }
        };
        self
    }

    /// Initialize the verify configuration options.
    pub fn verify(&mut self, verify: Option<ConfigOptsVerify>) -> &mut Self {
        self.verify = verify;
        self
    }

    /// Initialize the API token.
    pub fn verify_token(
        &mut self,
        token: Option<impl Into<String>>,
    ) -> &mut Self {
        self.verify.get_or_insert_default().token = token.map(|t| t.into());
        self
    }

    /// Initialize the list configuration options.
    pub fn list(&mut self, list: Option<ConfigOptsList>) -> &mut Self {
        self.list = list;
        self
    }

    /// Initialize the include zones.
    pub fn list_include_zones(
        &mut self,
        include_zones: Option<Vec<String>>,
    ) -> &mut Self {
        self.list.get_or_insert_default().include_zones = include_zones;
        self
    }

    /// Initialize the ignore zones.
    pub fn list_ignore_zones(
        &mut self,
        ignore_zones: Option<Vec<String>>,
    ) -> &mut Self {
        self.list.get_or_insert_default().ignore_zones = ignore_zones;
        self
    }

    /// Initialize the include records.
    pub fn list_include_records(
        &mut self,
        include_records: Option<Vec<String>>,
    ) -> &mut Self {
        self.list.get_or_insert_default().include_records = include_records;
        self
    }

    /// Initialize the ignore records.
    pub fn list_ignore_records(
        &mut self,
        ignore_records: Option<Vec<String>>,
    ) -> &mut Self {
        self.list.get_or_insert_default().ignore_records = ignore_records;
        self
    }

    /// Initialize the inventory configuration options.
    pub fn inventory(
        &mut self,
        inventory: Option<ConfigOptsInventory>,
    ) -> &mut Self {
        self.inventory = inventory;
        self
    }

    /// Initialize the inventory path.
    pub fn inventory_path(&mut self, path: Option<PathBuf>) -> &mut Self {
        self.inventory.get_or_insert_default().path = path;
        self
    }

    /// Initialize the inventory force update flag.
    pub fn inventory_force_update(&mut self, force: Option<bool>) -> &mut Self {
        self.inventory.get_or_insert_default().force_update = force;
        self
    }

    /// Initialize the inventory force prune flag.
    pub fn inventory_force_prune(&mut self, force: Option<bool>) -> &mut Self {
        self.inventory.get_or_insert_default().force_prune = force;
        self
    }

    /// Initialize the inventory watch interval.
    pub fn inventory_watch_interval(
        &mut self,
        interval: Option<u64>,
    ) -> &mut Self {
        self.inventory.get_or_insert_default().watch_interval = interval;
        self
    }

    /// Build an configuration options model.
    pub fn build(&self) -> ConfigOpts {
        ConfigOpts {
            verify: {
                let verify = self.verify.as_ref();
                ConfigOptsVerify {
                    token: verify.and_then(|o| o.token.clone()),
                }
            },
            list: {
                let list = self.list.as_ref();
                ConfigOptsList {
                    include_zones: list.and_then(|o| o.include_zones.clone()),
                    ignore_zones: list.and_then(|o| o.ignore_zones.clone()),
                    include_records: list
                        .and_then(|o| o.include_records.clone()),
                    ignore_records: list.and_then(|o| o.ignore_records.clone()),
                }
            },
            inventory: {
                let inventory = self.inventory.as_ref();
                ConfigOptsInventory {
                    path: inventory.and_then(|o| o.path.clone()),
                    force_update: inventory.and_then(|o| o.force_update),
                    force_prune: inventory.and_then(|o| o.force_prune),
                    watch_interval: inventory.and_then(|o| o.watch_interval),
                }
            },
        }
    }

    /// Save the config file at the given path, overwriting if necessary.
    pub async fn save(&self, path: impl AsRef<Path>) -> Result<()> {
        let toml = crate::io::encoding::as_toml(&self)?;
        crate::io::fs::save(path, toml).await?;
        Ok(())
    }
}

impl From<ConfigOpts> for ConfigBuilder {
    fn from(opts: ConfigOpts) -> Self {
        Self {
            verify: Some(opts.verify),
            list: Some(opts.list),
            inventory: Some(opts.inventory),
        }
    }
}

impl From<Option<ConfigOpts>> for ConfigBuilder {
    fn from(opts: Option<ConfigOpts>) -> Self {
        match opts {
            None => Self {
                verify: None,
                list: None,
                inventory: None,
            },
            Some(o) => o.into(),
        }
    }
}
