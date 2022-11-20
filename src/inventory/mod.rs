//! CDDNS inventory management.
//!
//! CDDNS uses YAML to format inventory files.
//! Below is an example:
//! ```yaml
//! # You can use Cloudflare IDs
//! 9aad55f2e0a8d9373badd4361227cabe:
//!   - 5dba009abaa3ba5d3a624e87b37f941a
//! # Or Cloudflare names
//! imbleau.com:
//!   - *.imbleau.com
//! ```
pub mod models;

/// Return the default inventory path.
pub fn default_inventory_path() -> std::path::PathBuf {
    std::path::PathBuf::from("inventory.yaml")
}
