//! Cloudflare API gateway.
//!
//! This module exports API endpoints to interface the Cloudflare API.
//! Learn more: https://api.cloudflare.com

/// The stable base URL for all Version 4 HTTPS endpoints to Cloudflare.
pub const API_BASE: &str = "https://api.cloudflare.com/client/v4/";

pub mod endpoints;
pub mod models;
pub mod requests;
