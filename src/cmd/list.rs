use std::path::PathBuf;

use crate::{
    cloudfare,
    config::{ConfigOpts, ConfigOptsList},
};
use anyhow::Result;
use clap::{Args, Subcommand};

/// List Cloudfare resources
#[derive(Debug, Args)]
#[clap(name = "list")]
pub struct List {
    #[clap(subcommand)]
    action: Option<ListSubcommands>,
    #[clap(flatten)]
    pub cfg: ConfigOptsList,
}

impl List {
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let toml_cfg = ConfigOpts::from_file(config)?;
        let env_cfg = ConfigOpts::from_env()?;
        let cli_cfg = ConfigOpts {
            list: Some(self.cfg),
            ..Default::default()
        };
        // Apply layering to configuration data (TOML < ENV < CLI)
        let opts = toml_cfg.merge(env_cfg).merge(cli_cfg);

        let token = match opts.verify.map(|opts| opts.token).flatten() {
            Some(t) => t,
            None => anyhow::bail!("no token was provided"),
        };

        match self.action {
            Some(filter) => match filter {
                ListSubcommands::Zones => {
                    print_zones(&token).await?;
                }
                ListSubcommands::Records => {
                    print_records(&token).await?;
                }
            },
            None => {
                print_all(&token).await?;
            }
        }

        Ok(())
    }
}

async fn print_all(token: &str) -> Result<()> {
    print_zones(token).await?;
    print_records(token).await?;
    Ok(())
}

async fn print_zones(token: &str) -> Result<()> {
    println!("{:#?}", cloudfare::endpoints::zones(token).await?);
    Ok(())
}

async fn print_records(token: &str) -> Result<()> {
    println!("{:#?}", cloudfare::endpoints::records(token).await?);
    Ok(())
}

#[derive(Clone, Debug, Subcommand)]
enum ListSubcommands {
    /// Show zones (domains, subdomains, and identities)
    Zones,
    /// Show authoritative DNS records
    Records,
}
