use crate::{
    cloudfare,
    config::models::{ConfigOpts, ConfigOptsVerify},
};
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Verify authentication to Cloudfare.
#[derive(Debug, Args)]
#[clap(name = "verify")]
pub struct VerifyCmd {
    #[clap(flatten)]
    pub cfg: ConfigOptsVerify,
}

impl VerifyCmd {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            verify: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);

        if let Some(token) = opts.verify.map(|opts| opts.token).flatten() {
            println!("Verifying...");
            let login_messages = cloudfare::endpoints::verify(&token).await?;
            if let Some(message_stack) = login_messages
                .into_iter()
                .map(|msg| msg.message)
                .reduce(|cur: String, nxt: String| cur + "\n" + &nxt)
            {
                println!("{}", message_stack);
            }
            Ok(())
        } else {
            anyhow::bail!("no token was provided")
        }
    }
}
