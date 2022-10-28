use crate::config::{ConfigOpts, ConfigOptsVerify, DEFAULT_CONFIG_PATH};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{
    ffi::OsString,
    io::Write,
    path::{Path, PathBuf},
};

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

async fn prompt(
    prompt: &str,
    stdin_rx: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<String> {
    std::io::stdout().write(format!("{}: > ", prompt).as_bytes())?;
    std::io::stdout().flush()?;

    let input = tokio::select! {
        Some(line) = stdin_rx.recv() => {
            match line.as_str() {
                "exit" | "quit" => {
                    None
                },
                _ => {
                    Some(line)
                }
            }
        }
    };

    Ok(input.context("aborted")?)
}

async fn save<T, P>(
    contents: &T,
    default_path: P,
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    let name = default_path
        .as_ref()
        .file_name()
        .expect("invalid default path file name");
    let ext = default_path
        .as_ref()
        .extension()
        .expect("invalid default file extension");
    // Get output path
    let mut output_path = OsString::from(
        prompt(
            &format!(
                "Output path [default: {}]",
                default_path.as_ref().display()
            ),
            receiver,
        )
        .await?,
    );
    if output_path.is_empty() {
        output_path.push(name);
    }
    let output_path = &PathBuf::from(&output_path).with_extension(ext);
    if output_path.exists() {
        match prompt("File location exists, overwrite? (y/N)", receiver)
            .await?
            .to_lowercase()
            .as_str()
        {
            "y" | "yes" => {
                tokio::fs::remove_file(output_path).await?;
            }
            _ => anyhow::bail!("aborted"),
        };
    }

    // Save
    tokio::fs::write(
        output_path,
        toml::to_string(&contents).context("error encoding config to TOML")?,
    )
    .await
    .context("error saving contents")?;
    Ok(())
}

async fn build_config(
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()> {
    // Build config
    let token = prompt("Cloudfare API token", receiver).await?;
    let config = ConfigOpts {
        verify: Some(ConfigOptsVerify { token: Some(token) }),
        list: None,
    };

    save(&config, DEFAULT_CONFIG_PATH, receiver).await?;
    println!("Saved");
    Ok(())
}
