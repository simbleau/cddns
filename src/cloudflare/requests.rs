use crate::cloudflare::models::CloudflareResponse;
use crate::cloudflare::API_BASE;
use anyhow::{anyhow, Context, Result};
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
        .context("error sending HTTP request")?
        .bytes()
        .await
        .context("error retrieving HTTP response bytes")?;

    let cf_resp: CloudflareResponse = serde_json::from_slice(bytes.as_slice())
        .context("error deserializing cloudflare metadata")?;
    match cf_resp.success {
        true => Ok(serde_json::from_slice(bytes.as_slice())
            .context("error deserializing cloudflare payload")?),
        false => {
            let mut context_chain = anyhow!("unsuccessful cloudflare status");
            for err in cf_resp.errors {
                context_chain = context_chain.context(format!("error {}", err));
                while let Some(ref messages) = err.error_chain {
                    for message in messages {
                        context_chain = context_chain
                            .context(format!("  - error {}", message));
                    }
                }
            }
            Err(context_chain)
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
        .context("error sending HTTP request")?
        .bytes()
        .await
        .context("error retrieving HTTP response bytes")?;
    let cf_resp: CloudflareResponse = serde_json::from_slice(bytes.as_slice())
        .context("error deserializing cloudflare metadata")?;
    match cf_resp.success {
        true => Ok(serde_json::from_slice(bytes.as_slice())
            .context("error deserializing cloudflare payload")?),
        false => {
            let mut context_chain = anyhow!("unsuccessful cloudflare status");
            for err in cf_resp.errors {
                context_chain = context_chain.context(format!("error {}", err));
                while let Some(ref messages) = err.error_chain {
                    for message in messages {
                        context_chain = context_chain
                            .context(format!("  - error {}", message));
                    }
                }
            }
            Err(context_chain)
        }
    }
}
