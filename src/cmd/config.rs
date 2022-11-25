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
    #[tracing::instrument(level = "trace", skip(self, config))]
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        match self.action {
            ConfigSubcommands::Build => build().await,
            ConfigSubcommands::Show => show(config).await,
        }
    }
}

#[tracing::instrument(level = "trace")]
async fn build() -> Result<()> {
    let runtime = tokio::runtime::Handle::current();
    let mut scanner = Scanner::new(runtime);

    // Prompt
    println!("Welcome! This builder will build a CLI configuration file without needing to understand TOML.");
    println!("For annotated examples of each field, please visit https://github.com/simbleau/cddns/blob/main/config.toml");
    println!("You can skip any field for configuration defaults via enter (no answer.)");
    println!();
    let token = scanner.prompt("Cloudflare API token", "string").await?;
    let include_zones = scanner
        .prompt_ron("Include zone filters, e.g. `[\".*.com\"]`", "list[string]")
        .await?;
    let ignore_zones = scanner
        .prompt_ron(
            "Ignore zone filters, e.g. `[\"ex1.com\", \"ex2.com\"]`",
            "list[string]",
        )
        .await?;
    let include_records = scanner
        .prompt_ron(
            "Include record filters, e.g. `[\"shop.imbleau.com\"]`",
            "list[string]",
        )
        .await?;
    let ignore_records = scanner
        .prompt_ron("Ignore record filters, e.g. `[]`", "list[string]")
        .await?;
    let path = scanner
        .prompt_t::<PathBuf>("Inventory path", "path")
        .await?;
    let force = scanner
        .prompt_yes_or_no("Force on `inventory commit`?", "y/N")
        .await?
        .unwrap_or(false);
    let interval = scanner
        .prompt_t::<u64>(
            "Interval for `inventory watch`, in milliseconds",
            "number",
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
        .prompt_t::<PathBuf>(
            format!("Save location [default: {}]", default_path.display()),
            "path",
        )
        .await?
        .map(|p| match p.extension() {
            Some(_) => p,
            None => p.with_extension("toml"),
        })
        .unwrap_or(default_path);
    io::fs::remove_interactive(&path, &mut scanner).await?;
    config.save(path).await?;

    Ok(())
}

#[tracing::instrument(level = "trace")]
async fn show(config: Option<PathBuf>) -> Result<()> {
    let cfg = ConfigOpts::full(config, None)?;
    println!("{}", cfg);
    Ok(())
}
