use axum::{
    Router, middleware as axum_middleware,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
pub mod middleware;
mod routes;
mod state;

use config::AppConfig;
pub use middleware::static_files;
use routes::{auth, tasks};
use state::AppState;

#[tokio::main]
async fn main() {
    let log_dir = std::env::var("LOG_DIR").ok().or_else(|| {
        let data_dir = std::path::Path::new("/app/data");
        if data_dir.is_dir() {
            Some("/app/data/log".to_string())
        } else {
            Some("/app/log".to_string())
        }
    });

    let (file_layer_error, file_layer_app) = if let Some(ref dir) = log_dir {
        if dir == "off" || dir == "none" || dir == "false" {
            (None, None)
        } else {
            let _ = std::fs::create_dir_all(dir);
            let error_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("app.log"))
                .ok();

            let error_layer = error_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::WARN)
            });

            let app_layer = app_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
            });

            (error_layer, app_layer)
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer_error)
        .with(file_layer_app)
        .init();

    let config = AppConfig::load();

    // Create data storage and tasks.json
    tasks::initialize_storage();

    let asset_manifest = std::sync::Arc::new(static_files::build_asset_manifest());
    let state = AppState::new(config.clone(), asset_manifest);

    // Rate-limit cleanup thread.
    //
    // Note: login-attempt lockouts are now tracked in
    // `shared_backend::auth::attempts` (process-global). Entries self-expire
    // on read in `is_locked_out`, so no cleanup thread is required for them.
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state_clone.clean_old_rate_limits().await;
        }
    });

    // Use the canonical CORS layer from shared-assets. The previous inline
    // version only allowed GET and POST; the shared version correctly
    // allows all common REST methods.
    let server_config: Arc<shared_backend::server::ServerConfig> = Arc::new(config.server.clone());
    let cors = shared_backend::middleware::cors_layer(&server_config);

    let api_routes = Router::new()
        .route(
            "/tasks",
            get(tasks::get_tasks).post(tasks::save_tasks).layer(
                axum_middleware::from_fn_with_state(state.clone(), auth::require_pin),
            ),
        )
        .route("/verify-pin", post(auth::verify_pin))
        .route("/logout", post(auth::logout))
        .route(
            "/auth-check",
            get(auth::auth_check).layer(axum_middleware::from_fn_with_state(
                state.clone(),
                auth::require_pin,
            )),
        )
        .route("/pin-required", get(auth::pin_required))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth::rate_limit_middleware,
        ))
        .layer(axum_middleware::from_fn_with_state(
            state.clone(),
            auth::origin_validation_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        .route("/health", get(tasks::serve_health))
        .route("/favicon.svg", get(static_files::serve_favicon))
        .route("/favicon.png", get(static_files::serve_favicon_png))
        .route("/manifest.json", get(static_files::serve_manifest))
        .route(
            "/asset-manifest.json",
            get(static_files::serve_asset_manifest),
        )
        .route(
            "/service-worker.js",
            get(static_files::serve_service_worker),
        )
        .route("/", get(tasks::serve_index))
        .route("/index.html", get(tasks::serve_index))
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(axum_middleware::from_fn(
            shared_backend::middleware::security_headers_layer,
        ))
        .layer(axum_middleware::from_fn_with_state(
            shared_backend::middleware::HstsState(server_config.clone()),
            shared_backend::middleware::hsts_layer,
        ))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
