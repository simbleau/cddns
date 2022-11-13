use crate::{
    config::{
        default_config_path,
        models::{ConfigOpts, ConfigOptsVerify},
    },
    io::{self, Scanner},
};
use anyhow::{Context, Result};
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
                // TODO: Prompt until answer
                let token = scanner
                    .prompt("Cloudflare API token")
                    .await?
                    .context("empty answer not allowed")?;

                // Package config
                let config = ConfigOpts {
                    verify: Some(ConfigOptsVerify { token: Some(token) }),
                    ..Default::default()
                };

                // Save
                let default_path = default_config_path()
                    .unwrap_or_else(|| PathBuf::from("config.toml"));
                let path = scanner
                    .prompt_path(format!(
                        "Save location [default: {}]",
                        default_path.display()
                    ))
                    .await?
                    .map(|p| match p.extension() {
                        Some(_) => p,
                        None => p.with_extension("toml"),
                    })
                    .unwrap_or(default_path);
                if path.exists() {
                    io::fs::remove_interactive(&path, &mut scanner).await?;
                }
                io::fs::save_toml(&config, &path).await?;
                println!("✅ Saved");
            }
            ConfigSubcommands::Show => {
                // TODO: Need to show this in a better format.
                let cfg = ConfigOpts::full(config, None)?;
                println!("{:#?}", cfg);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Subcommand)]
enum ConfigSubcommands {
    /// Build the CDDNS configuration file.
    Build,
    /// Show the current config pre-CLI.
    Show,
}
