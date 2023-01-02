use anyhow::{Context, Result};

/// Serialize an object to TOML.
pub fn as_toml<T>(contents: &T) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    toml::to_string(&contents).context("encoding as TOML")
}

/// Serialize an object to YAML.
pub fn as_yaml<T>(contents: &T) -> Result<String>
where
    T: ?Sized + serde::Serialize,
{
    serde_yaml::to_string(&contents).context("encoding as YAML")
}
