use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The model for DNS record inventory.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Inventory(Option<HashMap<String, InventoryZone>>);

/// The model for a zone with records.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryZone(Option<Vec<InventoryRecord>>);

/// The model for a DNS record.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct InventoryRecord(String);
