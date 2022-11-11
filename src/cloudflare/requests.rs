use crate::cloudflare::models::CloudfareResponse;
use crate::cloudflare::API_BASE;
use anyhow::{Context, Result};
use core::slice::SlicePattern;
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Display;

pub async fn get<T>(endpoint: impl Display, token: impl Display) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = reqwest::Client::new()
        .get(format!("{}{}", API_BASE, endpoint))
        .bearer_auth(token)
        .send()
        .await
        .context("sending HTTP request")?
        .bytes()
        .await
        .context("retrieving HTTP response")?;

    let cf_resp: CloudfareResponse = serde_json::from_slice(bytes.as_slice())
        .context("reading cloudfare response")?;
    match cf_resp.success {
        true => Ok(serde_json::from_slice(bytes.as_slice())
            .context("deserializing cloudfare payload")?),
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
                            // TODO: Make this cleaner
                            .as_ref()
                            .unwrap_or(&vec![])
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

pub async fn patch<T>(
    endpoint: impl Display,
    token: impl Display,
    json: &(impl Serialize + ?Sized),
) -> Result<T>
where
    T: DeserializeOwned,
{
    let bytes = reqwest::Client::new()
        .patch(format!("{}{}", API_BASE, endpoint))
        .bearer_auth(token)
        .header("Content-Type", "application/json")
        .json(json)
        .send()
        .await
        .context("sending HTTP request")?
        .bytes()
        .await
        .context("retrieving HTTP response")?;
    let cf_resp: CloudfareResponse = serde_json::from_slice(bytes.as_slice())
        .context("reading cloudfare response")?;
    match cf_resp.success {
        true => Ok(serde_json::from_slice(bytes.as_slice())
            .context("deserializing cloudfare payload")?),
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
                            // TODO: Make this cleaner
                            .as_ref()
                            .unwrap_or(&vec![])
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
