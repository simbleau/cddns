use std::path::PathBuf;

use crate::{
    cloudfare,
    config::{ConfigOpts, ConfigOptsVerify},
};
use anyhow::Result;
use clap::Args;

/// Verify authentication
#[derive(Debug, Args)]
#[clap(name = "verify")]
pub struct Verify {
    #[clap(flatten)]
    pub cfg: ConfigOptsVerify,
}

impl Verify {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            verify: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);

        println!("{:#?}", opts);
        let verify_opts = opts.verify.unwrap_or_default();
        let token = match verify_opts.token {
            Some(t) => t,
            None => anyhow::bail!("no token was provided"),
        };
        Ok(cloudfare::endpoints::verify(&token).await?)
    }
}
