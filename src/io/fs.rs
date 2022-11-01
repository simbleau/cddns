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
    let overwrite = loop {
        match scanner.prompt("Location exists, overwrite? [y/N]").await? {
            Some(input) => match input.to_lowercase().as_str() {
                "y" | "yes" => break true,
                "" | "n" | "no" | "exit" | "quit" => break false,
                _ => continue,
            },
            None => break false,
        }
    };
    if overwrite {
        remove_force(path).await?;
    } else {
        anyhow::bail!("aborted")
    }
    Ok(())
}

pub async fn save_toml<T, P>(contents: &T, path: P) -> Result<()>
where
    T: ?Sized + serde::Serialize,
    P: AsRef<Path>,
{
    tokio::fs::write(
        match path.as_ref().extension() {
            Some(_) => path.as_ref().to_owned(),
            None => path.as_ref().with_extension("toml"),
        },
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
        match path.as_ref().extension() {
            Some(_) => path.as_ref().to_owned(),
            None => path.as_ref().with_extension("yaml"),
        },
        serde_yaml::to_string(&contents).context("error encoding to YAML")?,
    )
    .await
    .context("error saving YAML contents")?;
    Ok(())
}
