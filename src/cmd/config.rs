use crate::{
    config::{
        ConfigOpts, ConfigOptsInventory, ConfigOptsList, ConfigOptsVerify,
        DEFAULT_CONFIG_PATH,
    },
    io::{self, Scanner},
};
use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

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

                // Build
                let token = scanner
                    .prompt("Cloudfare API token")
                    .await?
                    .unwrap_or_default();

                // Package config
                let config = ConfigOpts {
                    verify: Some(ConfigOptsVerify { token: Some(token) }),
                    list: Some(ConfigOptsList::default()),
                    inventory: Some(ConfigOptsInventory::default()),
                };

                // Save
                let path = scanner
                    .prompt_path_or(
                        format!(
                            "Save location [default: {}]",
                            DEFAULT_CONFIG_PATH
                        ),
                        DEFAULT_CONFIG_PATH.into(),
                    )
                    .await?;
                if path.exists() {
                    io::fs::remove_interactive(&path, &mut scanner).await?;
                }
                io::fs::save_toml(&config, &path).await?;
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
