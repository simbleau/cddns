use crate::{
    cloudfare::{
        self,
        models::{Record, Zone},
    },
    config::{ConfigOpts, ConfigOptsList},
};
use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use regex::Regex;
use std::{collections::BTreeMap, path::PathBuf};
use tokio::io;

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
        let (stdin_tx, stdin_rx) = tokio::sync::mpsc::channel(1);
        start_reading(stdin_tx, tokio::runtime::Handle::current());

        let x = read_input(stdin_rx).await;
        println!("Got {:?}", x);
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
            let line = line_buf.trim_end().to_string();
            line_buf.clear();
            {
                let sender = sender.clone();
                runtime.spawn(async move { sender.send(line).await });
            }
        }
    });
}

async fn read_input(
    mut stdin_rx: tokio::sync::mpsc::Receiver<String>,
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
