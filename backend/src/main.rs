use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path as StdPath;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::prelude::*;

static START_TIME: OnceLock<Instant> = OnceLock::new();

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub site_title: String,
    pub apprise_url: Option<String>,
    pub apprise_message: String,
    pub pin: Option<String>,
}

impl AppConfig {
    pub fn load() -> Self {
        dotenvy::dotenv().ok();
        let port = std::env::var("PORT")
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(4405);
        let site_title = std::env::var("SITE_TITLE").unwrap_or_else(|_| "RustKan".to_string());
        let apprise_url = std::env::var("APPRISE_URL").ok().filter(|s| !s.is_empty());
        let apprise_message = std::env::var("APPRISE_MESSAGE")
            .unwrap_or_else(|_| "Kanban Board updated: {action}".to_string());
        let pin = std::env::var("RUSTKAN_PIN")
            .or_else(|_| std::env::var("PIN"))
            .ok()
            .filter(|p| {
                !p.is_empty()
                    && p.chars().all(|c| c.is_ascii_digit())
                    && p.len() >= 4
                    && p.len() <= 10
            });
        Self {
            port,
            site_title,
            apprise_url,
            apprise_message,
            pin,
        }
    }
}

mod static_files;

#[derive(Clone)]
pub struct AppState {
    config: AppConfig,
    client: reqwest::Client,
    pub asset_manifest: std::sync::Arc<Vec<String>>,
}

#[tokio::main]
async fn main() {
    START_TIME.set(Instant::now()).ok();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::load();
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build reqwest client");

    // Initialize data storage directory and default tasks.json
    initialize_storage();

    let asset_manifest = std::sync::Arc::new(static_files::build_asset_manifest());

    let state = AppState {
        config: config.clone(),
        client,
        asset_manifest,
    };

    let cors = get_cors_layer();

    let api_routes = Router::new()
        .route("/pin-required", get(pin_required))
        .route("/verify-pin", post(verify_pin))
        .route("/logout", post(logout))
        .route("/auth-check", get(auth_check))
        .route("/tasks", get(get_tasks).post(save_tasks))
        .layer(middleware::from_fn(origin_validation_middleware));

    let app = Router::new()
        .nest("/api", api_routes)
        // Backwards compatible task endpoints for the frontend
        .route("/data/tasks.json", get(get_tasks).post(save_tasks))
        .route("/health", get(serve_health))
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
        .route("/", get(serve_index))
        .route("/index.html", get(serve_index))
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(cors)
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], config.port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

fn initialize_storage() {
    if !StdPath::new("data").exists() {
        let _ = fs::create_dir_all("data");
    }
    let tasks_path = StdPath::new("data/tasks.json");
    if !tasks_path.exists() {
        let default_structure = serde_json::json!({
            "boards": {
                "work": {
                    "name": "Work",
                    "columns": {
                        "todo": { "name": "To Do", "tasks": [] },
                        "doing": { "name": "Doing", "tasks": [] },
                        "done": { "name": "Done", "tasks": [] }
                    }
                }
            },
            "activeBoard": "work"
        });
        let _ = fs::write(
            tasks_path,
            serde_json::to_string_pretty(&default_structure).unwrap(),
        );
    }
}

// Health Handler
async fn serve_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339(),
        "uptime": START_TIME.get().map(|t| t.elapsed().as_secs()).unwrap_or(0)
    }))
}

// Serve index.html and perform dynamic replacement of title
async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = StdPath::new("frontend/dist/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let rendered = content.replace("{{SITE_TITLE}}", &state.config.site_title);
            Html(rendered).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

// Verification Handlers
async fn pin_required(State(state): State<AppState>) -> impl IntoResponse {
    Json(serde_json::json!({
        "required": state.config.pin.is_some(),
        "length": state.config.pin.as_ref().map(|p| p.len()).unwrap_or(0),
    }))
}

#[derive(Deserialize)]
struct VerifyPinPayload {
    pin: Option<String>,
}

async fn verify_pin(
    State(state): State<AppState>,
    Json(payload): Json<VerifyPinPayload>,
) -> impl IntoResponse {
    let Some(ref config_pin) = state.config.pin else {
        let mut headers = axum::http::header::HeaderMap::new();
        headers.insert(
            axum::http::header::SET_COOKIE,
            axum::http::header::HeaderValue::from_static(
                "RUSTKAN_PIN=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
            ),
        );
        return (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response();
    };

    let pin_str = payload.pin.as_deref().unwrap_or("").trim();
    if pin_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "PIN is required." })),
        )
            .into_response();
    }

    if safe_compare(pin_str, config_pin) {
        let mut headers = axum::http::header::HeaderMap::new();
        headers.insert(
            axum::http::header::SET_COOKIE,
            axum::http::header::HeaderValue::from_str(&format!(
                "RUSTKAN_PIN={}; Path=/; HttpOnly; SameSite=Lax",
                pin_str
            ))
            .unwrap(),
        );
        (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response()
    } else {
        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({ "success": false, "error": "Invalid PIN" })),
        )
            .into_response()
    }
}

async fn logout() -> impl IntoResponse {
    let mut headers = axum::http::header::HeaderMap::new();
    headers.insert(
        axum::http::header::SET_COOKIE,
        axum::http::header::HeaderValue::from_static(
            "RUSTKAN_PIN=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
        ),
    );
    (
        StatusCode::OK,
        headers,
        Json(serde_json::json!({ "success": true })),
    )
        .into_response()
}

async fn auth_check(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(ref pin) = state.config.pin {
        if !is_authorized(&headers, pin) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }
    StatusCode::OK.into_response()
}

// Tasks GET/POST
async fn get_tasks(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(ref pin) = state.config.pin {
        if !is_authorized(&headers, pin) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    match tokio::fs::read_to_string("data/tasks.json").await {
        Ok(data) => (StatusCode::OK, data).into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

async fn save_tasks(
    headers: axum::http::HeaderMap,
    State(state): State<AppState>,
    Json(payload): Json<serde_json::Value>,
) -> impl IntoResponse {
    if let Some(ref pin) = state.config.pin {
        if !is_authorized(&headers, pin) {
            return StatusCode::UNAUTHORIZED.into_response();
        }
    }

    match tokio::fs::write(
        "data/tasks.json",
        serde_json::to_string_pretty(&payload).unwrap(),
    )
    .await
    {
        Ok(_) => {
            // Trigger Apprise notification
            if state.config.apprise_url.is_some() {
                let action = determine_action(&payload);
                trigger_apprise_notification(&action, &state.config, &state.client).await;
            }
            Json(serde_json::json!({ "ok": true })).into_response()
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

// Helpers
fn is_authorized(headers: &axum::http::HeaderMap, pin: &str) -> bool {
    let cookie_pin = headers
        .get(axum::http::header::COOKIE)
        .and_then(|c| c.to_str().ok())
        .and_then(|c_str| {
            c_str
                .split(';')
                .find(|s| s.trim().starts_with("RUSTKAN_PIN="))
                .and_then(|s| s.split('=').nth(1))
                .map(|s| s.trim().to_string())
        });
    let header_pin = headers
        .get("x-pin")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    let provided_pin = cookie_pin.or(header_pin);

    match provided_pin {
        Some(prov) => safe_compare(&prov, pin),
        None => false,
    }
}

fn safe_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut result = 0;
    for (x, y) in a.bytes().zip(b.bytes()) {
        result |= x ^ y;
    }
    result == 0
}

fn determine_action(payload: &serde_json::Value) -> String {
    // Return a descriptive update state
    let active_board = payload
        .get("activeBoard")
        .and_then(|v| v.as_str())
        .unwrap_or("work");
    format!("Board '{}' state modified", active_board)
}

async fn trigger_apprise_notification(action: &str, config: &AppConfig, client: &reqwest::Client) {
    let url = match &config.apprise_url {
        Some(u) => u,
        None => return,
    };

    let message = config.apprise_message.replace("{action}", action);

    let body = serde_json::json!({
        "urls": url,
        "body": message,
        "title": format!("{} Notification", config.site_title),
    });

    tracing::info!("Sending notification via Apprise to URL: {}", url);
    let _ = client
        .post("https://api.apprise.io/notify")
        .json(&body)
        .send()
        .await;
}

fn get_cors_layer() -> CorsLayer {
    use axum::http::HeaderValue;
    use tower_http::cors::Any;

    let origins_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    if origins_env == "*" {
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
    } else {
        let mut origins = Vec::new();
        for origin in origins_env.split(',') {
            let o = origin.trim();
            if !o.is_empty() {
                if let Ok(val) = HeaderValue::from_str(o) {
                    origins.push(val);
                }
            }
        }
        CorsLayer::new()
            .allow_origin(origins)
            .allow_methods(Any)
            .allow_headers(Any)
    }
}

async fn origin_validation_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<Response, StatusCode> {
    let origins_env = std::env::var("ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
    if origins_env == "*" {
        return Ok(next.run(req).await);
    }

    let referer = req.headers().get("referer").and_then(|v| v.to_str().ok());
    let host = req.headers().get("host").and_then(|v| v.to_str().ok());

    let origin = if let Some(ref_val) = referer {
        if let Ok(url) = reqwest::Url::parse(ref_val) {
            url.origin().ascii_serialization()
        } else {
            ref_val.to_string()
        }
    } else if let Some(host_val) = host {
        let proto = req
            .headers()
            .get("x-forwarded-proto")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("http");
        format!("{}://{}", proto, host_val)
    } else {
        return Err(StatusCode::FORBIDDEN);
    };

    let allowed_list: Vec<String> = origins_env
        .split(',')
        .map(|s| {
            let s_trim = s.trim();
            if let Ok(url) = reqwest::Url::parse(s_trim) {
                url.origin().ascii_serialization()
            } else {
                s_trim.to_string()
            }
        })
        .collect();

    let normalized_origin = if let Ok(url) = reqwest::Url::parse(&origin) {
        url.origin().ascii_serialization()
    } else {
        origin.clone()
    };

    if allowed_list.contains(&normalized_origin) {
        Ok(next.run(req).await)
    } else {
        tracing::warn!("Blocked request from origin: {}", origin);
        Err(StatusCode::FORBIDDEN)
    }
}
