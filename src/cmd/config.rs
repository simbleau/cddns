use crate::{
    config::{
        default_config_path,
        models::{
            ConfigOpts, ConfigOptsCommit, ConfigOptsInventory, ConfigOptsList,
            ConfigOptsVerify, ConfigOptsWatch,
        },
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

#[derive(Clone, Debug, Subcommand)]
enum ConfigSubcommands {
    /// Build a configuration file.
    Build,
    /// Show the current configuration.
    Show,
}

impl ConfigCmd {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        match self.action {
            ConfigSubcommands::Build => build().await,
            ConfigSubcommands::Show => show(config).await,
        }
    }
}

async fn build() -> Result<()> {
    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);

    // Prompt
    let token = scanner
        .prompt("Cloudflare API token [default: skip]")
        .await?;
    let include_zones = scanner
        .prompt("Ignore zone filters [default: skip]")
        .await?
        .map(|s| s.split_terminator(' ').map(str::to_owned).collect());
    let ignore_zones = scanner
        .prompt("Ignore zone filters [default: skip]")
        .await?
        .map(|s| s.split(' ').map(str::to_owned).collect());
    let include_records = scanner
        .prompt("Include record filters [default: skip]")
        .await?
        .map(|s| s.split(' ').map(str::to_owned).collect());
    let ignore_records = scanner
        .prompt("Ignore record filters [default: skip]")
        .await?
        .map(|s| s.split(' ').map(str::to_owned).collect());
    let path = scanner
        .prompt_t::<PathBuf>("Inventory path [default: skip]")
        .await?;
    let force = !scanner
        .prompt_yes_or_no("Prompt for permission for `inventory commit` [Y/n]")
        .await?
        .unwrap_or(true);
    let interval = scanner
        .prompt_t::<u64>(
            "Interval for `inventory watch` in milliseconds [default: skip]",
        )
        .await?;

    // Build
    let config = ConfigOpts {
        verify: Some(ConfigOptsVerify { token }),
        list: Some(ConfigOptsList {
            include_zones,
            ignore_zones,
            include_records,
            ignore_records,
        }),
        inventory: Some(ConfigOptsInventory { path }),
        commit: Some(ConfigOptsCommit { force }),
        watch: Some(ConfigOptsWatch { interval }),
    };

    // Save
    let default_path =
        default_config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
    let path = scanner
        .prompt_t::<PathBuf>(format!(
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

    Ok(())
}

async fn show(config: Option<PathBuf>) -> Result<()> {
    // TODO: Need to show this in a better format.
    let cfg = ConfigOpts::full(config, None)?;
    println!("{:#?}", cfg);
    Ok(())
}
