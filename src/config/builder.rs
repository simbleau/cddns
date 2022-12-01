use crate::config::models::{
    ConfigOpts, ConfigOptsCommit, ConfigOptsInventory, ConfigOptsList,
    ConfigOptsVerify, ConfigOptsWatch,
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
    pub commit: Option<ConfigOptsCommit>,
    pub watch: Option<ConfigOptsWatch>,
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

    /// Initialize the verify token.
    pub fn verify_token(
        &mut self,
        token: Option<impl Into<String>>,
    ) -> &mut Self {
        self.verify.get_or_insert_default().token = token.map(|t| t.into());
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

    /// Build an configuration options model.
    pub fn build(self) -> ConfigOpts {
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
            commit: ConfigOptsCommit {
                force: self
                    .commit
                    .map(|o| o.force)
                    .unwrap_or(ConfigOptsCommit::default().force),
            },
            watch: ConfigOptsWatch {
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
