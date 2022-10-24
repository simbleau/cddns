use crate::cloudfare::API_BASE;
use anyhow::{Context, Result};
use serde::de::DeserializeOwned;

pub(crate) async fn get<T>(endpoint: &'static str, token: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let resp = reqwest::Client::new()
        .get(format!("{}{}", API_BASE, endpoint))
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .context("error sending HTTPS request")?
        .json()
        .await
        .context("error parsing HTTPS content")?;
    Ok(resp)
}
