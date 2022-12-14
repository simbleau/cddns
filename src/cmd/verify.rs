use crate::cloudflare;
use crate::config::models::{ConfigOpts, ConfigOptsVerify};
use anyhow::{Context, Result};
use clap::Args;
use tracing::info;

/// Verify authentication to Cloudflare.
#[derive(Debug, Args)]
#[clap(name = "verify")]
pub struct VerifyCmd {
    #[clap(flatten)]
    pub cfg: ConfigOptsVerify,
}

impl VerifyCmd {
    #[tracing::instrument(level = "trace", skip_all)]
    pub async fn run(self, opts: ConfigOpts) -> Result<()> {
        // Apply CLI configuration layering
        let cli_opts = ConfigOpts::builder().verify(Some(self.cfg)).build();
        let opts = ConfigOpts::builder().merge(opts).merge(cli_opts).build();

        // Run
        verify(&opts).await
    }
}

#[tracing::instrument(level = "trace", skip_all)]
async fn verify(opts: &ConfigOpts) -> Result<()> {
    info!("verifying, please wait...");
    // Get token
    let token = opts
        .verify.token.as_ref()
        .context("no token was provided, need help? see https://github.com/simbleau/cddns#readme")?;
    // Get response
    let cf_messages = cloudflare::endpoints::verify(token)
        .await
        .context("verification failure, need help? see https://github.com/simbleau/cddns#readme")?;
    // Log responses
    for (i, response) in cf_messages.iter().enumerate() {
        info!(response = i + 1, response.message);
    }
    info!("verification complete");
    Ok(())
}
