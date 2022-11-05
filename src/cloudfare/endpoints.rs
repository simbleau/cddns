use std::fmt::Display;

use crate::cloudfare::models::{
    ListRecordsResponse, ListZonesResponse, Record, VerifyResponse, Zone,
};
use crate::cloudfare::requests;
use anyhow::{Context, Result};

/// Return Ok if the token is verifiable
pub async fn verify(token: &str) -> Result<()> {
    let resp: VerifyResponse = requests::get("/user/tokens/verify", token)
        .await
        .context("error verifying API token")?;
    if let Some(message_stack) = resp
        .messages
        .into_iter()
        .map(|msg| msg.message)
        .reduce(|cur: String, nxt: String| cur + "\n" + &nxt)
    {
        println!("{}", message_stack);
    }
    Ok(())
}

/// Return all known Cloudfare zones
pub async fn zones(token: impl Display) -> Result<Vec<Zone>> {
    let mut zones = vec![];
    let mut page_cursor = 1;
    loop {
        let endpoint = format!("/zones?order=name&page={}", page_cursor);
        let resp: ListZonesResponse =
            requests::get(endpoint, token.to_string())
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
pub async fn records(
    zones: &Vec<Zone>,
    token: impl Display,
) -> Result<Vec<Record>> {
    let mut records = vec![];
    for zone in zones {
        let mut page_cursor = 1;
        loop {
            let endpoint = format!(
                "/zones/{}/dns_records?order=name&page={}",
                zone.id, page_cursor
            );
            let resp: ListRecordsResponse =
                requests::get(endpoint, token.to_string())
                    .await
                    .context("error resolving records endpoint")?;
            anyhow::ensure!(resp.success, "error retrieving records for zone");

            records.extend(resp.result.into_iter().filter(|record| {
                record.record_type == "A"
                    || record.record_type == "AAAA" && record.locked == false
            }));

            page_cursor += 1;
            if page_cursor > resp.result_info.total_pages {
                break;
            }
        }
    }
    Ok(records)
}
