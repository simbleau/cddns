use crate::inventory::DEFAULT_INVENTORY_PATH;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};

/// The model for DNS record inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory(pub Option<HashMap<String, InventoryZone>>);

/// The model for a zone with records.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryZone(pub Option<Vec<InventoryRecord>>);

/// The model for a DNS record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryRecord(pub String);

impl Inventory {
    /// Read inventory from a target path.
    pub fn from_file(path: Option<PathBuf>) -> Result<Self> {
        let mut inventory_path = path.unwrap_or(DEFAULT_INVENTORY_PATH.into());
        anyhow::ensure!(inventory_path.exists(), "inventory was not found");
        if !inventory_path.is_absolute() {
            inventory_path =
                inventory_path.canonicalize().with_context(|| {
                    format!(
                        "error getting canonical path to inventory file {:?}",
                        &inventory_path
                    )
                })?;
        }
        let cfg_bytes = std::fs::read(&inventory_path)
            .context("error reading inventory file")?;
        let cfg: Self = serde_yaml::from_slice(&cfg_bytes)
            .context("error reading inventory file contents as YAML data")?;
        Ok(cfg)
    }
}
