use crate::io::Scanner;
use anyhow::{Context, Result};
use std::path::Path;
use tracing::debug;

/// If a file exists, remove it by force without user interaction
pub async fn remove_force(path: impl AsRef<Path>) -> Result<()> {
    if path.as_ref().exists() {
        tokio::fs::remove_file(path.as_ref()).await?;
        debug!("removed: {}", path.as_ref().display());
    }
    Ok(())
}

/// If a file exists, remove it only after user grants permission
pub async fn remove_interactive(
    path: impl AsRef<Path>,
    scanner: &mut Scanner,
) -> Result<()> {
    if path.as_ref().exists() {
        let overwrite = scanner
            .prompt_yes_or_no(
                format!("Path '{}' exists, remove?", path.as_ref().display()),
                "y/N",
            )
            .await?
            .unwrap_or(false);
        if overwrite {
            remove_force(path).await?;
        } else {
            anyhow::bail!("aborted")
        }
    }
    Ok(())
}

/// Save a serializable object as a TOML file, creating directories if
/// necessary.
pub async fn save_toml<T>(contents: &T, path: impl AsRef<Path>) -> Result<()>
where
    T: ?Sized + serde::Serialize,
{
    if let Some(parent) = path.as_ref().parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("could not create parent directory")?;
    }
    tokio::fs::write(
        path.as_ref(),
        toml::to_string(&contents).context("encoding to TOML")?,
    )
    .await
    .context("saving TOML contents")?;
    debug!("wrote: {}", path.as_ref().display());
    Ok(())
}

/// Save a serializable object as a YAML file, creating directories if
/// necessary.
pub async fn save_yaml<T>(contents: &T, path: impl AsRef<Path>) -> Result<()>
where
    T: ?Sized + serde::Serialize,
{
    if let Some(parent) = path.as_ref().parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .context("could not create parent directory")?;
    }
    tokio::fs::write(
        path.as_ref(),
        serde_yaml::to_string(&contents).context("encoding to YAML")?,
    )
    .await
    .context("saving YAML contents")?;
    debug!("wrote: {}", path.as_ref().display());
    Ok(())
}
