use crate::config::{ConfigOpts, ConfigOptsInventory};
use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

/// Verify authentication to Cloudfare
#[derive(Debug, Args)]
#[clap(name = "inventory")]
pub struct Inventory {
    #[clap(flatten)]
    pub cfg: ConfigOptsInventory,
}

impl Inventory {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            inventory: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);

        Ok(())
    }
}
