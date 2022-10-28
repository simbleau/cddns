use crate::config::{ConfigOpts, ConfigOptsVerify};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::io::Write;

/// Build configuration or inventory files
#[derive(Debug, Args)]
#[clap(name = "build")]
pub struct Build {
    #[clap(subcommand)]
    action: BuildSubcommands,
}

#[derive(Clone, Debug, Subcommand)]
enum BuildSubcommands {
    /// Build a CLI config file
    Config,
    /// Build a DNS record inventory
    Inventory,
}

impl Build {
    pub async fn run(self) -> Result<()> {
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel(1);
        start_reading(stdin_tx, tokio::runtime::Handle::current());

        match self.action {
            BuildSubcommands::Config => build_config(&mut stdin_rx).await?,
            BuildSubcommands::Inventory => todo!(),
        };
        Ok(())
    }
}

fn start_reading(
    sender: tokio::sync::mpsc::Sender<String>,
    runtime: tokio::runtime::Handle,
) {
    std::thread::spawn(move || {
        let stdin = std::io::stdin();
        let mut line_buf = String::new();
        while stdin.read_line(&mut line_buf).is_ok() {
            let line = line_buf.trim().to_string();
            line_buf.clear();
            {
                let sender = sender.clone();
                runtime.spawn(async move { sender.send(line).await });
            }
        }
    });
}

async fn read_input(
    stdin_rx: &mut tokio::sync::mpsc::Receiver<String>,
) -> Option<String> {
    tokio::select! {
        Some(line) = stdin_rx.recv() => {
            match line.as_str() {
                "exit" => {
                    None
                },
                _ => {
                    Some(line)
                }
            }
        }
    }
}

async fn user_input(
    prompt: &str,
    stdin_rx: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<String> {
    std::io::stdout().write(format!("{}: > ", prompt).as_bytes())?;
    std::io::stdout().flush()?;

    Ok(read_input(stdin_rx).await.context("Aborted")?)
}

async fn build_config(
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()> {
    let token = user_input("Cloudfare API token", receiver)
        .await
        .context("Aborted")?;

    let config = ConfigOpts {
        verify: Some(ConfigOptsVerify { token: Some(token) }),
        list: None,
    };

    println!(
        "{}",
        toml::to_string(&config).context("error encoding config")?
    );

    Ok(())
}
