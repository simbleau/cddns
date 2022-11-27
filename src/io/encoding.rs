use crate::{
    cloudflare::{
        self,
        models::{Record, Zone},
    },
    config::models::ConfigOpts,
    inventory::models::InventoryData,
};
use anyhow::{Context, Result};
use tracing::{debug, trace, warn};

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

pub struct InventoryPostProcessor {
    zones: Vec<Zone>,
    records: Vec<Record>,
}

impl InventoryPostProcessor {
    pub async fn try_init(opts: &ConfigOpts) -> Result<Self> {
        let token = opts
                    .verify
                    .as_ref()
                    .and_then(|opts| opts.token.clone())
                    .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;
        debug!("retrieving post-processing resources...");
        let zones = cloudflare::endpoints::zones(&token).await?;
        let records = cloudflare::endpoints::records(&zones, &token).await?;
        let pp = InventoryPostProcessor::from(zones, records);
        Ok(pp)
    }

    pub fn from(zones: Vec<Zone>, records: Vec<Record>) -> Self {
        Self { zones, records }
    }
}

impl PostProcessor for InventoryPostProcessor {
    /// Annotate an inventory with zone and record ID/Name comments.
    fn post_process(&self, yaml: &mut String) -> Result<()> {
        let data = serde_yaml::from_slice::<InventoryData>(yaml.as_bytes())
            .context("deserializing inventory from bytes")?;

        for (zone_id, record_ids) in data.into_iter() {
            // Post-process zone
            if let Some(zone) =
                crate::cmd::list::find_zone(&self.zones, &zone_id)
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
                    crate::cmd::list::find_record(&self.records, &record_id)
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
