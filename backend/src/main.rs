use axum::{
    Router, middleware,
    routing::{get, post},
};
use std::net::SocketAddr;
use std::time::Duration;
use tower_http::services::ServeDir;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod auth;
mod config;
mod handlers;
mod state;
mod static_files;
mod utils;

use config::AppConfig;
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
                .write(true)
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
                .write(true)
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
    handlers::initialize_storage();

    let asset_manifest = std::sync::Arc::new(static_files::build_asset_manifest());
    let state = AppState::new(config.clone(), asset_manifest);

    // Lockout cleanup thread
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state_clone.clean_old_lockouts().await;
        }
    });

    let cors = if config.allowed_origins == "*" {
        tower_http::cors::CorsLayer::permissive()
    } else {
        let mut cors = tower_http::cors::CorsLayer::new()
            .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
            .allow_headers([axum::http::header::CONTENT_TYPE, axum::http::header::COOKIE]);
        for origin in config.allowed_origins.split(',') {
            if let Ok(parsed) = origin.trim().parse::<axum::http::HeaderValue>() {
                cors = cors.allow_origin(parsed);
            }
        }
        cors.allow_credentials(true)
    };

    let api_routes = Router::new()
        .route(
            "/tasks",
            get(handlers::get_tasks).post(handlers::save_tasks).layer(
                middleware::from_fn_with_state(state.clone(), auth::require_pin),
            ),
        )
        .route("/verify-pin", post(auth::verify_pin))
        .route("/logout", post(auth::logout))
        .route(
            "/auth-check",
            get(auth::auth_check).layer(middleware::from_fn_with_state(
                state.clone(),
                auth::require_pin,
            )),
        )
        .route("/pin-required", get(auth::pin_required))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::origin_validation_middleware,
        ));

    let app = Router::new()
        .nest("/api", api_routes)
        .route(
            "/data/tasks.json",
            get(handlers::get_tasks).post(handlers::save_tasks).layer(
                middleware::from_fn_with_state(state.clone(), auth::require_pin),
            ),
        )
        .route("/health", get(handlers::serve_health))
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
        .route("/", get(handlers::serve_index))
        .route("/index.html", get(handlers::serve_index))
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(middleware::from_fn(auth::security_headers_middleware))
        .layer(cors)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();
}
