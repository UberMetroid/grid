//! Grid-specific configuration layered on top of shared [`ServerConfig`].

use shared_backend::server::ServerConfig;

/// Grid application configuration. Wraps [`ServerConfig`] (added in
/// shared-assets v3.0.0) with any grid-specific settings.
#[derive(Clone, Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
}

impl AppConfig {
    pub fn load() -> Self {
        Self {
            server: ServerConfig::from_env("GRID"),
        }
    }
}
