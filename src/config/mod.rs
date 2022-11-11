//! CDDNS config.
//!
//! CDDNS takes the typical layered configuration approach. There are 3 layers.
//! The config file is the base, which is then superseded by environment
//! variables, which are finally superseded by CLI arguments and options.

/// The default location to configuration.
/// TODO: $XDG_CONFIG_HOME/cddns/config.toml
pub const DEFAULT_CONFIG_PATH: &str = "CFDDNS.toml";

pub mod models;
