use crate::cloudfare::{models::VerifyResponse, requests};
use anyhow::Result;

/// Return true if the token is verified
pub async fn verify(token: &str) -> Result<()> {
    let resp: VerifyResponse =
        requests::get("/user/tokens/verify", token).await?;
    anyhow::ensure!(
        resp.success,
        "API token verification failed. Check `cfddns config show`."
    );
    Ok(())
}
