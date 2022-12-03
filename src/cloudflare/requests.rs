use crate::cloudflare::models::CloudflareResponse;
use crate::cloudflare::API_BASE;
use anyhow::{anyhow, Context, Result};
use core::slice::SlicePattern;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Display, future::Future, time::Duration};
use tokio::time::error::Elapsed;
use tracing::trace;

async fn timeout<T>(future: T) -> Result<<T>::Output, Elapsed>
where
    T: Future,
{
    tokio::time::timeout(Duration::from_millis(10_000), future).await
}

pub async fn get<T>(endpoint: impl Display, token: impl Display) -> Result<T>
where
    T: DeserializeOwned,
{
    trace!("starting http request");
    let bytes = reqwest::Client::new()
        .get(format!("{}{}", API_BASE, endpoint))
        .bearer_auth(token)
        .send()
        .await
        .context("error sending HTTP request")?
        .bytes()
        .await
        .context("error retrieving HTTP response bytes")?;
    trace!("received http response");

    let cf_resp: CloudflareResponse = serde_json::from_slice(bytes.as_slice())
        .context("error deserializing cloudflare metadata")?;
    match cf_resp.success {
        true => Ok(serde_json::from_slice(bytes.as_slice())
            .context("error deserializing cloudflare payload")?),
        false => {
            let mut context_chain = anyhow!("unsuccessful cloudflare status");
            for err in cf_resp.errors {
                context_chain = context_chain.context(format!("error {}", err));
                if let Some(ref messages) = err.error_chain {
                    for message in messages {
                        context_chain =
                            context_chain.context(format!("error {}", message));
                    }
                }
            }
            Err(context_chain)
        }
    }
}

pub async fn get_with_timeout<T>(
    endpoint: impl Display,
    token: impl Display,
) -> Result<T>
where
    T: DeserializeOwned,
{
    timeout(get(endpoint, token))
        .await
        .context("request to cloudflare timed out")?
}

pub async fn patch<T>(
    endpoint: impl Display,
    token: impl Display,
    json: &(impl Serialize + ?Sized),
) -> Result<T>
where
    T: DeserializeOwned,
{
    trace!("starting http request");
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
    trace!("received http response");

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

pub async fn patch_with_timeout<T>(
    endpoint: impl Display,
    token: impl Display,
    json: &(impl Serialize + ?Sized),
) -> Result<T>
where
    T: DeserializeOwned,
{
    timeout(patch(endpoint, token, json))
        .await
        .context("request to cloudflare timed out")?
}
