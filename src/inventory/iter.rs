use crate::inventory::models::InventoryData;
use std::collections::HashMap;

/// An iterator over the zones and corresponding records.
pub struct InventoryIter {
    items: Vec<(String, Vec<String>)>,
    curr: usize,
}

impl Iterator for InventoryIter {
    /// A tuple containing the zone ID and respective child record IDs
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

impl IntoIterator for InventoryData {
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
