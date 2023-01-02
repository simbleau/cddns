//! cddns inventory management.
//!
//! cddns uses YAML to format inventory files.
//! Below is an example:
//! ```yaml
//! # You can use Cloudflare IDs
//! 9aad55f2e0a8d9373badd4361227cabe:
//!   - 5dba009abaa3ba5d3a624e87b37f941a
//! # Or Cloudflare names
//! imbleau.com:
//!   - *.imbleau.com
//! ```

pub mod builder;
pub mod iter;
pub mod models;

/// Return the default inventory path, depending on the host OS.
///
/// - Linux: $XDG_CONFIG_HOME/cddns/inventory.yml or
///   $HOME/.config/cddns/inventory.yml
/// - MacOS: $HOME/Library/Application Support/cddns/inventory.yml
/// - Windows: {FOLDERID_RoamingAppData}/cddns/inventory.yml
/// - Else: ./inventory.yml
pub fn default_inventory_path() -> std::path::PathBuf {
    if let Some(base_dirs) = directories::BaseDirs::new() {
        let mut config_path = base_dirs.config_dir().to_owned();
        config_path.push("cddns");
        config_path.push("inventory.yml");
        config_path
    } else {
        std::path::PathBuf::from("inventory.yml")
    }
}
