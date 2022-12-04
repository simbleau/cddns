use crate::{
    config::{default_config_path, models::ConfigOpts},
    inventory::default_inventory_path,
    io::{
        self,
        scanner::{prompt, prompt_ron, prompt_t, prompt_yes_or_no},
    },
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
    // Prompt
    println!("Welcome! This builder will build a CLI configuration file without needing to understand TOML.");
    println!("For annotated examples of each field, please visit https://github.com/simbleau/cddns/blob/main/config.toml");
    println!("You can skip any answer for cddns' defaults, which may change over time.");

    // Build
    let mut builder = ConfigOpts::builder();
    builder
        .verify_token({
            println!();
            println!(r#"First provide your Cloudflare API token with permission to view and edit DNS records."#);
            println!(r#" > help? https://developers.cloudflare.com/fundamentals/api/get-started/create-token/"#);            
            println!(r#" > default: none"#);
            prompt("token", "string")?
        })
        .list_include_zones({
            println!();
            println!(r#"Next, if you want filtered ZONE output in the CLI, provide regex filters in RON notation which will INCLUDE output in `cddns inventory build` and `cddns list`."#);
            println!(r#" > what is RON? https://github.com/ron-rs/ron/wiki/Specification"#);
            println!(r#" > what are zones? https://www.cloudflare.com/learning/dns/glossary/dns-zone/"#);
            println!(r#" > examples: [], [".*.(com|dev)"], ["example.com", "example.dev"]"#);
            println!(r#" > default: [".*"] (all)"#);
            prompt_ron(
                "include zone filters",
                "list[string]",
            )?
        })
        .list_ignore_zones({
            println!();
            println!(r#"Next, if you want filtered ZONE output in the CLI, provide regex filters in RON notation which will IGNORE output in `cddns inventory build` and `cddns list`."#);
            println!(r#" > what is RON? https://github.com/ron-rs/ron/wiki/Specification"#);
            println!(r#" > what are zones? https://www.cloudflare.com/learning/dns/glossary/dns-zone/"#);
            println!(r#" > examples: [], [".*.(com|dev)"], ["example.com", "example.dev"]"#);
            println!(r#" > default: [] (none)"#);
            prompt_ron(
                "ignore zone filters",
                "list[string]",
            )?
        })
        .list_include_records({
            println!();
            println!(r#"Next, if you want filtered RECORD output in the CLI, provide regex filters in RON notation which will INCLUDE output in `cddns inventory build` and `cddns list`."#);
            println!(r#" > what is RON? https://github.com/ron-rs/ron/wiki/Specification"#);
            println!(r#" > what are records? https://www.cloudflare.com/learning/dns/dns-records/"#);
            println!(r#" > examples: [], [".*.example.com"], ["beta.example.com", "gamma.example.com"]"#);
            println!(r#" > default: [".*"] (all)"#);
            prompt_ron(
                "include record filters",
                "list[string]",
            )?
        })
        .list_ignore_records({
            println!();
            println!(r#"Next, if you want filtered RECORD output in the CLI, provide regex filters in RON notation which will IGNORE output in `cddns inventory build` and `cddns list`."#);
            println!(r#" > what is RON? https://github.com/ron-rs/ron/wiki/Specification"#);
            println!(r#" > what are records? https://www.cloudflare.com/learning/dns/dns-records/"#);
            println!(r#" > examples: [], [".*.example.com"], ["beta.example.com", "gamma.example.com"]"#);
            println!(r#" > default: [] (none)"#);
            prompt_ron("ignore record filters", "list[string]")?
        })
        .inventory_path({
            println!();
            println!(r#"Next provide the expected path for your DNS inventory file."#);
            println!(r#" > default: {}"#, default_inventory_path().display());
            prompt_t("inventory path", "path")?
        })
        .inventory_commit_force({
            println!();
            println!(r#"Next, would you like to force update and prune erroneous records when using the `inventory commit` command?"#);
            println!(r#" > default: no"#);
            prompt_yes_or_no("force on `inventory commit`?", "y/N")?
        })
        .inventory_watch_interval({
            println!();
            println!(r#"Next, specify the interval (in milliseconds) for DNS refresh when using `inventory watch`."#);
            println!(r#" > examples: 0 (continuously), 60000 (1 minute)"#);
            println!(r#" > default: 30000"#);
            prompt_t(
                "interval for `inventory watch`?",
                "number",
            )?
        });

    // Save
    let default_path =
        default_config_path().unwrap_or_else(|| PathBuf::from("config.toml"));
    let path = {
        println!();
        println!(r#"Finally, provide the save location for this config file."#);
        println!(r#" > default: {}"#, default_path.display());
        prompt_t::<PathBuf>(format!("Save location"), "path")?
            .map(|p| match p.extension() {
                Some(_) => p,
                None => p.with_extension("toml"),
            })
            .unwrap_or(default_path)
    };
    io::fs::remove_interactive(&path).await?;
    builder.save(path).await?;

    Ok(())
}

#[tracing::instrument(level = "trace")]
async fn show(config: Option<PathBuf>) -> Result<()> {
    println!("{}", ConfigOpts::full(config, None)?);
    Ok(())
}
