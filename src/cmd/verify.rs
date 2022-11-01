use std::path::PathBuf;

use crate::{
    cloudfare,
    config::{ConfigOpts, ConfigOptsVerify},
};
use anyhow::Result;
use clap::Args;

/// Verify authentication to Cloudfare
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
            Ok(cloudfare::endpoints::verify(&token).await?)
        } else {
            anyhow::bail!("no token was provided")
        }
    }
}
