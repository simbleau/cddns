use std::path::PathBuf;

use crate::config::{ConfigOpts, ConfigOptsCheck};
use anyhow::Result;
use clap::Args;

/// Perform a dry run for validation testing
#[derive(Debug, Args)]
#[clap(name = "check")]
pub struct Check {
    #[clap(flatten)]
    pub cfg: ConfigOptsCheck,
}

impl Check {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            check: Some(self.cfg),
        };
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);
        crate::cloudfare::endpoints::verify(&opts.check.unwrap().zone.unwrap())
            .await?;
        Ok(())
    }
}
