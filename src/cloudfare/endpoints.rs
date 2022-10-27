use crate::cloudfare::{
    models::{ListRecordsResponse, ListZonesResponse, VerifyResponse},
    requests,
};
use anyhow::{Context, Result};

use super::models::{Record, Zone};

/// Return Ok if the token is verifiable
pub async fn verify(token: &str) -> Result<()> {
    let resp: VerifyResponse = requests::get("/user/tokens/verify", token)
        .await
        .context("error verifying API token")?;
    println!("{:?}", resp.messages);
    Ok(())
}

/// Return all known Cloudfare zones
pub async fn zones(token: &str) -> Result<Vec<Zone>> {
    let mut zones = vec![];
    let mut page_cursor = 1;
    loop {
        let endpoint = format!("/zones?order=name&page={}", page_cursor);
        let resp: ListZonesResponse = requests::get(&endpoint, token)
            .await
            .context("error resolving zones endpoint")?;
        anyhow::ensure!(resp.success, "error retrieving zones");

        zones.extend(resp.result.into_iter().filter(|zone| {
            &zone.status == "active"
                && zone.permissions.contains(&"#zone:edit".to_string())
        }));

        page_cursor += 1;
        if page_cursor > resp.result_info.total_pages {
            break;
        }
    }
    Ok(zones)
}

/// Return all known Cloudfare records
pub async fn records(zones: &Vec<Zone>, token: &str) -> Result<Vec<Record>> {
    let mut records = vec![];
    for zone in zones {
        let mut page_cursor = 1;
        loop {
            let endpoint = format!(
                "/zones/{}/dns_records?order=name&page={}",
                zone.id, page_cursor
            );
            let resp: ListRecordsResponse = requests::get(&endpoint, token)
                .await
                .context("error resolving records endpoint")?;
            anyhow::ensure!(resp.success, "error retrieving records for zone");

            records.extend(resp.result.into_iter().filter(|record| {
                record.record_type == "A" && record.locked == false
            }));

            page_cursor += 1;
            if page_cursor > resp.result_info.total_pages {
                break;
            }
        }
    }
    Ok(records)
}
