//! Clap commands handled by the CLI.

mod config;
pub(crate) use config::ConfigCmd;
mod inventory;
pub(crate) use inventory::InventoryCmd;
mod list;
pub(crate) use list::ListCmd;
mod verify;
pub(crate) use verify::VerifyCmd;
