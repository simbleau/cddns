//! CFDDNS inventory management.
//!
//! CFDDNS uses YAML to format inventory files.
//! Below is an example:
//! ```yaml
//! # You can use Cloudfare IDs
//! 9aad55f2e0a8d9373badd4361227cabe:
//!   - 5dba009abaa3ba5d3a624e87b37f941a
//! # Or Cloudfare names
//! imbleau.com:
//!   - *.imbleau.com
//! ```

/// The default location to inventory.
pub const DEFAULT_INVENTORY_PATH: &str = "inventory.yaml";
/// The default interval for record checking.
pub const DEFAULT_WATCH_INTERVAL: u32 = 5000;

pub(crate) mod models;
