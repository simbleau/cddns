use std::path::PathBuf;

use anyhow::Result;
use clap::{Args, Subcommand};

use crate::{
    config::{
        ConfigOpts, ConfigOptsInventory, ConfigOptsList, ConfigOptsVerify,
        DEFAULT_CONFIG_PATH,
    },
    io::{self, Scanner},
};

/// Configuration controls
#[derive(Debug, Args)]
#[clap(name = "config")]
pub struct ConfigCmd {
    #[clap(subcommand)]
    action: ConfigSubcommands,
}

impl ConfigCmd {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        match self.action {
            ConfigSubcommands::Build => {
                let runtime = tokio::runtime::Handle::current();
                let mut scanner = Scanner::new(runtime);
                // Get user input
                let token = scanner.prompt("Cloudfare API token").await?;

                // Build config
                let config = ConfigOpts {
                    verify: Some(ConfigOptsVerify { token: Some(token) }),
                    list: Some(ConfigOptsList::default()),
                    inventory: Some(ConfigOptsInventory::default()),
                };

                // Save
                let path = scanner.prompt_path(DEFAULT_CONFIG_PATH).await?;
                io::fs::save_toml(&config, path).await?;
                println!("Saved");
            }
            ConfigSubcommands::Show => {
                let toml_cfg = ConfigOpts::from_file(config)?;
                let env_cfg = ConfigOpts::from_env()?;
                let cfg = toml_cfg.merge(env_cfg);
                println!("{:#?}", cfg);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
enum ConfigSubcommands {
    /// Build the CFDDNS configuration file.
    Build,
    /// Show the current config pre-CLI.
    Show,
}
