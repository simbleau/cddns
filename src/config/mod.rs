//! cddns configuration.
//!
//! cddns takes the typical layered configuration approach. There are 3 layers.
//! The config file is the base, which is then superseded by environment
//! variables, which are finally superseded by CLI arguments and options.

pub mod builder;
pub mod models;

/// Return the default configuration path, depending on the host OS.
///
/// - Linux: $XDG_CONFIG_HOME/cddns/config.toml or
///   $HOME/.config/cddns/config.toml
/// - MacOS: $HOME/Library/Application Support/cddns/config.toml
/// - Windows: {FOLDERID_RoamingAppData}/cddns/config.toml
/// - Else: ./config.toml
pub fn default_config_path() -> std::path::PathBuf {
    if let Some(base_dirs) = directories::BaseDirs::new() {
        let mut config_path = base_dirs.config_dir().to_owned();
        config_path.push("cddns");
        config_path.push("config.toml");
        config_path
    } else {
        std::path::PathBuf::from("config.toml")
    }
}
