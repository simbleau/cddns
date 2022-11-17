use crate::io::Scanner;
use anyhow::{Context, Result};
use std::path::Path;

/// Remove a file by force, without user interaction
pub async fn remove_force<P>(path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    if path.as_ref().exists() {
        tokio::fs::remove_file(&path).await?;
    }
    Ok(())
}

/// Remove a file only after user grants permission
pub async fn remove_interactive<P>(path: P, scanner: &mut Scanner) -> Result<()>
where
    P: AsRef<Path>,
{
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
    Ok(())
}

/// Save a serializable object as a TOML file.
pub async fn save_toml<T, P>(contents: &T, path: P) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        path,
        toml::to_string(&contents).context("encoding to TOML")?,
    )
    .await
    .context("saving TOML contents")?;
    Ok(())
}

/// Save a serializable object as a YAML file.
pub async fn save_yaml<T, P>(contents: &T, path: P) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        path,
        serde_yaml::to_string(&contents).context("encoding to YAML")?,
    )
    .await
    .context("saving YAML contents")?;
    Ok(())
}
