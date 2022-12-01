use crate::config::models::{
    ConfigOpts, ConfigOptsInventory, ConfigOptsInventoryCommit,
    ConfigOptsInventoryWatch, ConfigOptsList, ConfigOptsVerify,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A builder for configuration options.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigBuilder {
    pub verify: Option<ConfigOptsVerify>,
    pub list: Option<ConfigOptsList>,
    pub inventory: Option<ConfigOptsInventory>,
    pub commit: Option<ConfigOptsInventoryCommit>,
    pub watch: Option<ConfigOptsInventoryWatch>,
}

impl ConfigBuilder {
    /// Create a new config opts builder.
    pub fn new() -> Self {
        Self {
            verify: None,
            list: None,
            inventory: None,
            commit: None,
            watch: None,
        }
    }

    /// Merge config layers, where the `greater` layer takes precedence.
    pub fn merge(mut self, mut greater: impl Into<Self>) -> Self {
        let greater = greater.into();
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
                g.force = g.force.or(l.force);
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
    pub fn inventory_path(
        &mut self,
        path: Option<impl AsRef<Path>>,
    ) -> &mut Self {
        self.inventory.get_or_insert_default().path =
            path.map(|p| p.as_ref().to_owned());
        self
    }

    /// Initialize the inventory commit configuration options.
    pub fn inventory_commit(
        &mut self,
        inventory_commit: Option<ConfigOptsInventoryCommit>,
    ) -> &mut Self {
        self.commit = inventory_commit;
        self
    }

    /// Initialize the inventory commit force flag.
    pub fn inventory_commit_force(&mut self, force: Option<bool>) -> &mut Self {
        self.commit.get_or_insert_default().force = force;
        self
    }

    /// Initialize the inventory watch configuration options.
    pub fn inventory_watch(
        &mut self,
        inventory_watch: Option<ConfigOptsInventoryWatch>,
    ) -> &mut Self {
        self.watch = inventory_watch;
        self
    }

    /// Initialize the watch interval.
    pub fn inventory_watch_interval(
        &mut self,
        interval: Option<u64>,
    ) -> &mut Self {
        self.watch.get_or_insert_default().interval = interval;
        self
    }

    /// Build an configuration options model.
    pub fn build(&self) -> ConfigOpts {
        ConfigOpts {
            verify: ConfigOptsVerify {
                token: self.verify.and_then(|o| o.token),
            },
            list: ConfigOptsList {
                include_zones: self.list.and_then(|o| o.include_zones),
                ignore_zones: self.list.and_then(|o| o.ignore_zones),
                include_records: self.list.and_then(|o| o.include_records),
                ignore_records: self.list.and_then(|o| o.ignore_records),
            },
            inventory: ConfigOptsInventory {
                path: self.inventory.and_then(|o| o.path),
            },
            commit: ConfigOptsInventoryCommit {
                force: self.commit.and_then(|o| o.force),
            },
            watch: ConfigOptsInventoryWatch {
                interval: self.watch.and_then(|o| o.interval),
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
            commit: Some(opts.commit),
            watch: Some(opts.watch),
        }
    }
}
