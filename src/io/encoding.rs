use crate::{
    cloudflare::models::{Record, Zone},
    inventory::models::Inventory,
};
use anyhow::{Context, Result};
use tracing::{trace, warn};

/// Serialize an object to TOML.
pub fn as_toml<T>(contents: &T) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    toml::to_string(&contents).context("encoding as TOML")
}

pub fn as_inventory_yaml(
    inventory: &Inventory,
    zones: &Vec<Zone>,
    records: &Vec<Record>,
) -> Result<String> {
    let mut yaml = as_yaml(&inventory)?;

    // Post process comments
    trace!("beginning post-process of inventory file");
    for (zone_id, record_ids) in inventory.clone().into_iter() {
        // Post-process zone
        if let Some(zone) = crate::cmd::list::find_zone(zones, &zone_id) {
            let z_idx =
                yaml.find(&zone_id).context("zone not found in yaml")?;
            yaml.insert_str(
                z_idx + zone_id.len() + ":".len(),
                &format!(
                    " # '{}'",
                    if zone_id == zone.id {
                        zone.name
                    } else {
                        zone.id
                    }
                ),
            );
        } else {
            warn!(
                "post-processing '{}' failed: cloudflare zone not found",
                zone_id
            );
        }

        // Post-process records
        for record_id in record_ids {
            if let Some(record) =
                crate::cmd::list::find_record(records, &record_id)
            {
                let r_idx = yaml
                    .find(&record_id)
                    .context("record not found in yaml")?;
                yaml.insert_str(
                    r_idx + record_id.len(),
                    &format!(
                        " # '{}'",
                        if record_id == record.id {
                            record.name
                        } else {
                            record.id
                        }
                    ),
                );
            } else {
                warn!(
                    "post-processing '{}' failed: cloudflare record not found",
                    record_id
                );
            }
        }
    }
    trace!("finished post-processing of inventory file");

    Ok(yaml)
}

/// Serialize an object to YAML.
pub fn as_yaml<T>(contents: &T) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    serde_yaml::to_string(&contents).context("encoding as YAML")
}
