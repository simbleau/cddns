use crate::io::scanner::prompt_yes_or_no;
use anyhow::{bail, Context, Result};
use std::path::Path;
use tracing::debug;

/// If a file exists, remove it by force without user interaction.
pub async fn remove_force(path: impl AsRef<Path>) -> Result<()> {
    if path.as_ref().exists() {
        tokio::fs::remove_file(path.as_ref()).await?;
        debug!("removed: {}", path.as_ref().display());
    }
    Ok(())
}

/// If a file exists, remove it only after user grants permission.
pub async fn remove_interactive(path: impl AsRef<Path>) -> Result<()> {
    if path.as_ref().exists() {
        let overwrite = prompt_yes_or_no(
            format!("Path '{}' exists, remove?", path.as_ref().display()),
            "y/N",
        )?
        .unwrap_or(false);
        if overwrite {
            remove_force(path).await?;
        } else {
            bail!("aborted")
        }
    }
    Ok(())
}

/// Save the desired contents, overwriting and creating directories if
/// necessary.
pub async fn save(
    path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<()> {
    if let Some(parent) = path.as_ref().parent() {
        tokio::fs::create_dir_all(parent).await.with_context(|| {
            format!("unable to make directory '{}'", parent.display())
        })?;
    }
    if path.as_ref().exists() {
        debug!("overwriting '{}'...", path.as_ref().display());
        remove_force(path.as_ref()).await?;
    }
    tokio::fs::write(path.as_ref(), contents)
        .await
        .with_context(|| {
            format!("unable to write to '{}'", path.as_ref().display())
        })?;
    debug!("wrote: {}", path.as_ref().display());
    Ok(())
}
