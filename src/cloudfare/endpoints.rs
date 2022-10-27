use crate::cloudfare::{
    models::{ListZonesResponse, VerifyResponse},
    requests,
};
use anyhow::{Context, Result};

use super::models::Zone;

/// Return Ok if the token is verifiable
pub async fn verify(token: &str) -> Result<()> {
    let resp: VerifyResponse =
        requests::get("/user/tokens/verify", token).await?;
    anyhow::ensure!(resp.success, "error with verifying API token");
    Ok(())
}

/// Return all known Cloudfare zones
pub async fn zones(token: &str) -> Result<Vec<Zone>> {
    let mut zones = vec![];
    let mut page_cursor = 1;
    loop {
        let endpoint = format!("/zones?page={}", page_cursor);
        let resp: ListZonesResponse = requests::get(&endpoint, token).await?;
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
