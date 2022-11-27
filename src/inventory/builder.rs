use crate::inventory::models::Inventory;
use crate::inventory::models::InventoryData;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// A builder for Inventsory models.
#[derive(Default)]
pub struct InventoryBuilder {
    path: Option<PathBuf>,
    data: Option<InventoryData>,
}

impl InventoryBuilder {
    /// Create a new inventory builder.
    pub fn new() -> Self {
        Self {
            path: None,
            data: None,
        }
    }

    /// Initialize the inventory's path.
    pub fn path(mut self, path: impl AsRef<Path>) -> Self {
        self.path.replace(path.as_ref().to_owned());
        self
    }

    /// Initialize inventory with data.
    pub fn with_data(mut self, data: InventoryData) -> Self {
        self.data.replace(data);
        self
    }

    /// Initialize inventory data from bytes.
    pub fn with_contents<'a>(
        mut self,
        bytes: impl Into<&'a [u8]>,
    ) -> Result<Self> {
        self.data.replace(
            serde_yaml::from_slice(bytes.into())
                .context("deserializing inventory from bytes")?,
        );
        Ok(self)
    }

    /// Build an inventory model.
    pub fn build(self) -> Result<Inventory> {
        Ok(Inventory {
            path: self.path.context("uninitalized path")?,
            data: self.data.context("uninitialized inventory data")?,
        })
    }
}
