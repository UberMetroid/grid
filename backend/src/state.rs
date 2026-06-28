use crate::config::AppConfig;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

/// Application state.
///
/// Login-attempt tracking is no longer here — it lives in
/// `shared_backend::auth::attempts` as a process-global `OnceLock<Mutex<…>>`.
/// Keeping it global (rather than per-instance state) ensures that
/// concurrent requests on the same instance see consistent counters.
#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub asset_manifest: std::sync::Arc<Vec<String>>,
    pub active_sessions: Arc<RwLock<std::collections::HashSet<String>>>,
    pub rate_limiter: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
}

impl AppState {
    pub fn new(config: AppConfig, asset_manifest: std::sync::Arc<Vec<String>>) -> Self {
        Self {
            config,
            asset_manifest,
            active_sessions: Arc::new(RwLock::new(std::collections::HashSet::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sliding-window per-IP rate limiter, keyed by string IP.
    ///
    /// `ip` is expected to be the output of
    /// `shared_backend::server::get_client_ip`, which is already normalized
    /// (IPv6 mapped to IPv4 where applicable) and trusts `X-Forwarded-For`
    /// only when the connecting socket matches `TRUSTED_PROXY_IPS`.
    pub async fn check_rate_limit(&self, ip: &str) -> bool {
        const MAX_REQUESTS: usize = 100; // 100 requests
        const WINDOW: Duration = Duration::from_secs(60); // per 60 seconds
        let now = Instant::now();

        let mut map = self.rate_limiter.write().await;
        let timestamps = map.entry(ip.to_string()).or_insert_with(Vec::new);

        timestamps.retain(|&t| now.duration_since(t) < WINDOW);

        if timestamps.len() >= MAX_REQUESTS {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    pub async fn clean_old_rate_limits(&self) {
        const WINDOW: Duration = Duration::from_secs(60);
        let now = Instant::now();
        let mut map = self.rate_limiter.write().await;
        map.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < WINDOW);
            !timestamps.is_empty()
        });
    }
}
