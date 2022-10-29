use crate::{
    cloudfare::{self, models::Record},
    config::{
        ConfigOpts, ConfigOptsInventory, ConfigOptsList, ConfigOptsVerify,
        DEFAULT_CONFIG_PATH,
    },
    inventory::{
        Inventory, InventoryRecord, InventoryZone, DEFAULT_INVENTORY_PATH,
    },
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use std::{
    borrow::BorrowMut,
    collections::HashMap,
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
    pub async fn run(self, config: Option<PathBuf>) -> Result<()> {
        let (stdin_tx, mut stdin_rx) = tokio::sync::mpsc::channel(1);
        start_reading(stdin_tx, tokio::runtime::Handle::current());

        match self.action {
            BuildSubcommands::Config => build_config(&mut stdin_rx).await?,
            BuildSubcommands::Inventory => {
                build_inventory(config, &mut stdin_rx).await?
            }
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

    tokio::select! {
        Some(line) = stdin_rx.recv() => {
            match line.as_str() {
                "exit" | "quit" => {
                    Err(anyhow::anyhow!("aborted"))
                },
                _ => {
                    Ok(line)
                }
            }
        }
    }
}

async fn get_output_path<P>(
    default_path: P,
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<PathBuf>
where
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
    let output_path = PathBuf::from(&output_path).with_extension(ext);
    if output_path.exists() {
        match prompt("File location exists, overwrite? (y/N)", receiver)
            .await?
            .to_lowercase()
            .as_str()
        {
            "y" | "yes" => {
                tokio::fs::remove_file(&output_path).await?;
            }
            _ => anyhow::bail!("aborted"),
        };
    }
    Ok(output_path)
}

async fn save_toml<T, P>(
    contents: &T,
    default_path: P,
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        get_output_path(default_path, receiver).await?,
        toml::to_string(&contents).context("error encoding to TOML")?,
    )
    .await
    .context("error saving TOML contents")?;
    Ok(())
}

async fn save_yaml<T, P>(
    contents: &T,
    default_path: P,
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        get_output_path(default_path, receiver).await?,
        serde_yaml::to_string(&contents).context("error encoding to YAML")?,
    )
    .await
    .context("error saving YAML contents")?;
    Ok(())
}

async fn build_config(
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()> {
    // Build config
    let token = prompt("Cloudfare API token", receiver).await?;
    let config = ConfigOpts {
        verify: Some(ConfigOptsVerify { token: Some(token) }),
        list: Some(ConfigOptsList {
            ..Default::default()
        }),
        inventory: Some(ConfigOptsInventory {
            ..Default::default()
        }),
    };

    save_toml(&config, DEFAULT_CONFIG_PATH, receiver).await?;
    println!("Saved");
    Ok(())
}

async fn build_inventory(
    config: Option<PathBuf>,
    receiver: &mut tokio::sync::mpsc::Receiver<String>,
) -> Result<()> {
    let toml_cfg = ConfigOpts::from_file(config)?;
    let env_cfg = ConfigOpts::from_env()?;
    // Apply layering to configuration data (TOML < ENV < CLI)
    let opts = toml_cfg.merge(env_cfg);

    // Get token
    let token = opts
        .verify
        .as_ref()
        .map(|opts| opts.token.clone())
        .flatten()
        .context("no token was provided")?;

    let mut zones = cloudfare::endpoints::zones(&token).await?;
    crate::cmd::list::filter_zones(&mut zones, &opts)?;
    let mut records = cloudfare::endpoints::records(&zones, &token).await?;
    crate::cmd::list::filter_records(&mut records, &opts)?;

    let mut inventory = HashMap::new();
    'control: loop {
        anyhow::ensure!(zones.len() > 0, "no zones to build inventory from");
        let mut selection: Option<usize> = None;
        while selection.is_none() || selection.is_some_and(|i| *i > zones.len())
        {
            for (i, zone) in zones.iter().enumerate() {
                println!("[{}] {}: {}", i + 1, zone.name, zone.id);
            }
            match prompt("(1/2) Choose a zone", receiver).await {
                Ok(input) => selection = input.parse::<usize>().ok(),
                Err(_) => break 'control,
            }
        }
        let zone = &zones[selection.unwrap() - 1];
        let records = records
            .iter()
            .filter(|r| r.zone_id == zone.id)
            .collect::<Vec<&Record>>();
        if records.len() > 0 {
            selection = None;
            while selection.is_none()
                || selection.is_some_and(|i| *i > records.len())
            {
                for (i, record) in records.iter().enumerate() {
                    println!("[{}] {}: {}", i + 1, record.name, record.id);
                }
                match prompt("(2/2) Choose a record", receiver).await {
                    Ok(input) => selection = input.parse::<usize>().ok(),
                    Err(_) => break 'control,
                }
            }
            let record = &records[selection.unwrap() - 1];
            let key = zone.id.clone();
            let inventory_zone = inventory
                .entry(key)
                .or_insert_with(|| InventoryZone(Some(Vec::new())));
            inventory_zone
                .0
                .as_mut()
                .unwrap()
                .push(InventoryRecord(record.id.clone()));
            println!("Added {}: {}\n", record.name, record.id);
        }
    }

    let inventory = Inventory(Some(inventory));
    save_yaml(&inventory, DEFAULT_INVENTORY_PATH, receiver).await?;
    println!("Saved");
    Ok(())
}
