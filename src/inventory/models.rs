use crate::io::encoding::PostProcessor;
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
    path::Path,
};
use tracing::debug;

/// The model for DNS record inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory(Option<HashMap<String, InventoryZone>>);

/// The model for a zone with records.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryZone(Option<HashSet<InventoryRecord>>);

/// The model for a DNS record.
#[derive(Clone, Debug, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct InventoryRecord(String);

impl Inventory {
    /// Build a new inventory.
    pub fn new() -> Self {
        Self(None)
    }

    /// Read inventory from a string slice.
    pub fn from_str(contents: &str) -> Result<Self> {
        serde_yaml::from_slice(contents.as_bytes())
            .context("deserializing inventory contents from string")
    }

    /// Read inventory from a target path.
    pub async fn from_file(inventory_path: impl AsRef<Path>) -> Result<Self> {
        let path = inventory_path.as_ref();
        if !path.exists() {
            bail!("inventory file not found, need help? see https://github.com/simbleau/cddns#readme");
        } else {
            debug!("inventory file found");
        }
        let path = path.canonicalize().with_context(|| {
            format!(
                "getting canonical path to inventory file {:?}",
                path.display()
            )
        })?;
        debug!("reading inventory path: {}", path.display());
        let contents = tokio::fs::read_to_string(path)
            .await
            .context("reading inventory file")?;
        Self::from_str(&contents)
    }

    /// Save the inventory file at the given path, overwriting if necessary, and
    /// optionally with post-processed comments.
    pub async fn save<P>(
        &self,
        path: impl AsRef<Path>,
        post_processor: Option<P>,
    ) -> Result<()>
    where
        P: PostProcessor,
    {
        let yaml = crate::io::encoding::as_yaml(self, post_processor)?;
        crate::io::fs::save(path, yaml).await?;
        Ok(())
    }

    /// Returns whether a record exists.
    pub fn contains(
        &mut self,
        zone_id: impl Into<String>,
        record_id: impl Into<String>,
    ) -> bool {
        let zone_id = zone_id.into();
        let record_id = InventoryRecord(record_id.into());

        // Magic that checks whether the record exists
        self.0
            .as_ref()
            .and_then(|map| map.get(&zone_id))
            .and_then(|zone| zone.0.as_ref())
            .map(|records| records.contains(&record_id))
            .unwrap_or(false)
    }

    /// Insert a record into the inventory.
    pub fn insert(
        &mut self,
        zone_id: impl Into<String>,
        record_id: impl Into<String>,
    ) {
        let zone_id = zone_id.into();
        let record_id = record_id.into();

        // Magic that inserts the record
        self.0
            .get_or_insert(HashMap::new())
            .entry(zone_id)
            .or_insert_with(|| InventoryZone(None))
            .0
            .get_or_insert(HashSet::new())
            .insert(InventoryRecord(record_id));
    }

    /// Remove a record from the inventory. Returns whether the value was
    /// present in the set.
    pub fn remove(
        &mut self,
        zone_id: impl Into<String>,
        record_id: impl Into<String>,
    ) -> Result<bool> {
        let zone_id = zone_id.into();
        let record = InventoryRecord(record_id.into());

        // Magic that removes the record, returning true if the record was
        // present, false otherwise
        Ok(self
            .0
            .as_mut()
            .context("no zone map found")?
            .get_mut(&zone_id)
            .with_context(|| format!("no zone '{}'", zone_id))?
            .0
            .as_mut()
            .with_context(|| format!("no records in zone '{}'", zone_id))?
            .remove(&record))
    }

    /// Returns whether the inventory has no records
    pub fn is_empty(&self) -> bool {
        // Magic that checks whether there are records
        !self
            .0
            .as_ref()
            .map(|map| {
                map.iter().fold(0, |items, (_, zone)| {
                    items + zone.0.as_ref().map(|z| z.len()).unwrap_or(0)
                })
            })
            .is_some_and(|len| len > 0)
    }
}

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.clone() // TODO: Clone isn't necessary if traversed differently
                .into_iter()
                .map(|(zone, records)| {
                    format!(
                        "{}:{}",
                        zone,
                        records
                            .into_iter()
                            .map(|r| format!("\n  - {}", r))
                            .collect::<String>()
                    )
                })
                .intersperse("\n---\n".to_string())
                .collect::<String>()
        )
    }
}

/// An iterator over the zone and corresponding records.
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
        let mut items: HashMap<String, Vec<String>> = HashMap::new();
        if let Some(map) = self.0 {
            for (key, value) in map {
                let entry = items.entry(key.clone()).or_default();
                if let Some(record_set) = value.0 {
                    for record in record_set {
                        entry.push(record.0.clone());
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
