use crate::{
    cloudflare,
    config::models::{ConfigOpts, ConfigOptsVerify},
};
use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use tracing::info;

/// Verify authentication to Cloudflare.
#[derive(Debug, Args)]
#[clap(name = "verify")]
pub struct VerifyCmd {
    #[clap(flatten)]
    pub cfg: ConfigOptsVerify,
}

impl VerifyCmd {
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let cli_cfg = ConfigOpts {
            verify: Some(self.cfg),
            ..Default::default()
        };
        let opts = ConfigOpts::full(config, Some(cli_cfg))?;

        verify(&opts).await
    }
}

#[tracing::instrument(level = "trace", skip(opts))]
async fn verify(opts: &ConfigOpts) -> Result<()> {
    // Get token
    let token = opts
        .verify
        .as_ref()
        .and_then(|opts| opts.token.clone())
        .context("no token was provided")?;

    info!("Verifying...");
    let login_messages = cloudflare::endpoints::verify(&token).await?;
    if let Some(message_stack) = login_messages
        .into_iter()
        .map(|msg| msg.message)
        .reduce(|cur: String, nxt: String| cur + "\n" + &nxt)
    {
        println!("{}", message_stack);
    } else {
        println!("Token is valid.");
    }
    Ok(())
}
