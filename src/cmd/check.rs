use std::path::PathBuf;

use anyhow::Result;
use clap::Args;

/// Perform a dry run for validation testing
#[derive(Debug, Args)]
#[clap(name = "check")]
pub struct Check {}

impl Check {
    pub async fn run(self, _config: Option<PathBuf>) -> Result<()> {
        println!("Dry run");
        Ok(())
    }
}
