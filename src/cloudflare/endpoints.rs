use crate::cloudflare::models::{
    CloudfareMessage, ListRecordsResponse, ListZonesResponse,
    PatchRecordResponse, Record, VerifyResponse, Zone,
};
use crate::cloudflare::requests;
use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fmt::Display;
use tracing::{debug, trace};

/// Return a list of login messages if the token is verifiable.
pub async fn verify(token: &str) -> Result<Vec<CloudfareMessage>> {
    let resp: VerifyResponse = requests::get("/user/tokens/verify", token)
        .await
        .context("verifying API token")?;
    Ok(resp.messages)
}

/// Return all known Cloudfare zones.
pub async fn zones(token: impl Display) -> Result<Vec<Zone>> {
    let mut zones = vec![];
    let mut page_cursor = 1;
    loop {
        trace!("retrieving zones from page {}", page_cursor);
        let endpoint = format!("/zones?order=name&page={}", page_cursor);
        let resp: ListZonesResponse =
            requests::get(endpoint, token.to_string())
                .await
                .context("resolving zones endpoint")?;
        anyhow::ensure!(resp.success, "cloudfare response indicated failure");

        zones.extend(resp.result.into_iter().filter(|zone| {
            &zone.status == "active"
                && zone.permissions.contains(&"#zone:edit".to_string())
        }));

        page_cursor += 1;
        if page_cursor > resp.result_info.total_pages {
            break;
        }
    }
    debug!("collected {} zones", zones.len());
    Ok(zones)
}

/// Return all known Cloudfare records.
pub async fn records(
    zones: &Vec<Zone>,
    token: impl Display,
) -> Result<Vec<Record>> {
    let mut records = vec![];
    for zone in zones {
        trace!("retrieving records from zone '{}'", zone.id);
        let mut page_cursor = 1;
        let beginning_amt = records.len();
        loop {
            trace!("retrieving records from page {}", page_cursor);
            let endpoint = format!(
                "/zones/{}/dns_records?order=name&page={}",
                zone.id, page_cursor
            );
            let resp: ListRecordsResponse =
                requests::get(endpoint, token.to_string())
                    .await
                    .context("resolving records endpoint")?;
            anyhow::ensure!(
                resp.success,
                "cloudfare response indicated failure"
            );

            records.extend(resp.result.into_iter().filter(|record| {
                record.record_type == "A"
                    || record.record_type == "AAAA" && !record.locked
            }));

            page_cursor += 1;
            if page_cursor > resp.result_info.total_pages {
                break;
            }
        }
        debug!(
            "received {} records from zone '{}'",
            records.len() - beginning_amt,
            zone.id
        );
    }
    debug!("collected {} records", records.len());
    Ok(records)
}

/// Patch a cloudfare record.
pub async fn update_record(
    token: impl Display,
    zone_id: impl Display,
    record_id: impl Display,
    ip: impl Display,
) -> Result<()> {
    let endpoint = format!("/zones/{}/dns_records/{}", zone_id, record_id);

    let mut data = HashMap::new();
    data.insert("content", ip.to_string());

    let resp: PatchRecordResponse = requests::patch(endpoint, token, &data)
        .await
        .context("resolving records endpoint")?;
    anyhow::ensure!(resp.success, "cloudfare response indicated failure");
    Ok(())
}
