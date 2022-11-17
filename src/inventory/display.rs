use crate::inventory::models::Inventory;
use std::fmt::Display;

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.clone() // TODO: Clone isn't necessary if traversed
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
