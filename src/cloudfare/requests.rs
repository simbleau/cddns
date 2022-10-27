use crate::cloudfare::models::CloudfareResponse;
use crate::cloudfare::API_BASE;
use anyhow::{Context, Result};
use core::slice::SlicePattern;
use serde::de::DeserializeOwned;

pub(crate) async fn get<T>(endpoint: &str, token: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = reqwest::Client::new()
        .get(format!("{}{}", API_BASE, endpoint))
        .header("Content-Type", "application/json")
        .bearer_auth(token)
        .send()
        .await
        .context("error sending HTTPS request")?
        .bytes()
        .await
        .context("error retrieving web content")?;

    let cf_resp: CloudfareResponse =
        serde_json::from_slice(bytes.as_slice())
            .context("error reading cloudfare response")?;
    match cf_resp.success {
        true => Ok(serde_json::from_slice(bytes.as_slice())
            .context("error deserializing cloudfare payload")?),
        false => {
            if let Some(error_stack) = cf_resp
                .errors
                .iter()
                .map(|msg| {
                    format!(
                        "{}: {}{}",
                        msg.code,
                        msg.message,
                        msg.error_chain
                            .iter()
                            .map(|error| format!(
                                "\n  - {}: {}",
                                error.code, error.message
                            ))
                            .reduce(|cur: String, nxt: String| cur + &nxt)
                            .unwrap_or_default()
                    )
                })
                .reduce(|cur: String, nxt: String| cur + "\n" + &nxt)
            {
                Err(anyhow::anyhow!("{}", error_stack))
            } else {
                Err(anyhow::anyhow!("unknown error")
                    .context(format!("{:#?}", cf_resp)))
            }
        }
    }
}
