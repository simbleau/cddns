//! Cloudfare DDNS config.
//!
//! CFDDNS takes the typical layered configuration approach. There are 3 layers.
//! The `CFDDNS.toml` config file is the base, which is then superseded by
//! environment variables, which are finally superseded by CLI arguments and
//! options.

/// The default location to configuration.
pub const CONFIG_PATH: &str = "CFDDNS.toml";

mod models;
pub use models::*;
