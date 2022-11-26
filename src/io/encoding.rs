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

/// Serialize an object to YAML.
pub fn as_yaml<T>(
    contents: &T,
    post_processor: Option<impl PostProcessor>,
) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    let mut yaml =
        serde_yaml::to_string(&contents).context("encoding as YAML")?;
    // Post-processing
    if let Some(pp) = post_processor {
        trace!("beginning post-processing");
        pp.post_process(&mut yaml)?;
        trace!("finished post-processing");
    }
    Ok(yaml)
}

pub trait PostProcessor {
    fn post_process(&self, contents: &mut String) -> Result<()>;
}

pub struct InventoryPostProcessor<'p> {
    zones: &'p Vec<Zone>,
    records: &'p Vec<Record>,
}

impl<'p> InventoryPostProcessor<'p> {
    pub fn from(zones: &'p Vec<Zone>, records: &'p Vec<Record>) -> Self {
        Self { zones, records }
    }
}

impl<'p> PostProcessor for InventoryPostProcessor<'p> {
    /// Annotate an inventory with zone and record ID/Name comments.
    fn post_process(&self, yaml: &mut String) -> Result<()> {
        let inventory = Inventory::from_str(yaml)?;

        for (zone_id, record_ids) in inventory.into_iter() {
            // Post-process zone
            if let Some(zone) =
                crate::cmd::list::find_zone(self.zones, &zone_id)
            {
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
                    crate::cmd::list::find_record(self.records, &record_id)
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

        Ok(())
    }
}
