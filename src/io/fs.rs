use anyhow::{Context, Result};
use std::path::Path;

pub async fn save_toml<T, P>(contents: &T, path: P) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        path,
        toml::to_string(&contents).context("error encoding to TOML")?,
    )
    .await
    .context("error saving TOML contents")?;
    Ok(())
}

pub async fn save_yaml<T, P>(contents: &T, path: P) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        path,
        serde_yaml::to_string(&contents).context("error encoding to YAML")?,
    )
    .await
    .context("error saving YAML contents")?;
    Ok(())
}
