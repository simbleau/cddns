//! CFDDNS inventory management.
//!
//! CFDDNS takes uses YAML format for inventory files.
//! Below is an example:
//! ```yaml
//! imbleau.com: # A Cloudfare Zone
//!   - imbleau.com # A record under the imbleau.com zone
//!   - 5dba009abaa3ba5d3a624e87b37f941a # Using IDs also works
//! ```

/// The default location to inventory.
pub const DEFAULT_INVENTORY_PATH: &str = "inventory.yaml";

mod models;
pub use models::*;
