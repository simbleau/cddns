use crate::cloudfare::requests;
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Return true if the token is verified
pub async fn verify(token: &str) -> Result<()> {
    #[derive(Debug, Serialize, Deserialize)]
    struct Response {
        success: bool,
    }
    let resp: Response = requests::get("/user/tokens/verify", token).await?;
    anyhow::ensure!(
        resp.success,
        "API token verification failed. Check `cfddns config show`."
    );
    Ok(())
}
