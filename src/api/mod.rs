//! HTTP API layer: routes, handlers, and response models.

pub mod auth;
pub mod handlers;
pub mod models;
pub mod routes;
pub mod server;

use std::sync::Arc;

use crate::cache::{Cache, Coalescer};
use crate::client::GarupaClient;
use crate::config::Config;

/// Shared application state handed to every handler.
pub struct AppState {
    pub config: Config,
    pub client: GarupaClient,
    pub cache: Cache,
    pub coalescer: Coalescer,
}

pub type SharedState = Arc<AppState>;
