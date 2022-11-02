use crate::inventory::DEFAULT_INVENTORY_PATH;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};

/// The model for DNS record inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory(pub Option<HashMap<String, InventoryZone>>);

/// The model for a zone with records.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryZone(pub Option<HashSet<InventoryRecord>>);

/// The model for a DNS record.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
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

/// An iterator over the zone and corresponding records
pub struct InventoryIter {
    items: Vec<(String, Vec<String>)>,
    curr: usize,
}

impl Iterator for InventoryIter {
    /// A tuple containing the zone ID and a list of child record IDs
    type Item = (String, Vec<String>);

    fn next(&mut self) -> Option<Self::Item> {
        let current = self.curr;
        if current < self.items.len() {
            self.curr += 1;
            let (zone, records) = &self.items[current];
            Some((zone.clone(), records.clone()))
        } else {
            None
        }
    }
}

impl IntoIterator for Inventory {
    /// A tuple containing the zone ID and a list of child record IDs
    type Item = (String, Vec<String>);
    type IntoIter = InventoryIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut items = HashMap::new();
        if let Some(map) = self.0 {
            for (key, value) in map {
                items.entry(key.to_owned()).or_insert(Vec::new());
                if let Some(record_set) = value.0 {
                    for record in record_set {
                        items.get_mut(&key).unwrap().push(record.0.clone());
                    }
                }
            }
        }
        InventoryIter {
            items: Vec::from_iter(items),
            curr: 0,
        }
    }
}
