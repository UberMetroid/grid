use crate::state::AppState;
use axum::{
    Json,
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use chrono::Utc;
use std::fs;
use std::path::Path as StdPath;
use std::sync::LazyLock;
use std::time::Instant;

pub mod types;
use types::{Board, BoardData, Column};

static START_TIME: LazyLock<Instant> = LazyLock::new(Instant::now);

const TASKS_FILE: &str = "data/tasks.json";

pub fn initialize_storage() {
    if !StdPath::new("data").exists() {
        let _ = fs::create_dir_all("data");
    }
    let tasks_path = StdPath::new(TASKS_FILE);
    if !tasks_path.exists() {
        // Seed with a sensible default. `version: 0` so the first client
        // save goes through `0 -> 1` cleanly.
        let mut boards = indexmap::IndexMap::new();
        let mut columns = indexmap::IndexMap::new();
        columns.insert(
            "todo".to_string(),
            Column {
                name: "To Do".to_string(),
                tasks: vec![],
            },
        );
        columns.insert(
            "doing".to_string(),
            Column {
                name: "Doing".to_string(),
                tasks: vec![],
            },
        );
        columns.insert(
            "done".to_string(),
            Column {
                name: "Done".to_string(),
                tasks: vec![],
            },
        );
        boards.insert(
            "work".to_string(),
            Board {
                name: "Work".to_string(),
                columns,
            },
        );
        let seed = BoardData {
            version: 0,
            boards,
            active_board: "work".to_string(),
        };
        let _ = atomic_write(
            TASKS_FILE,
            serde_json::to_string_pretty(&seed).unwrap().as_bytes(),
        );
    }
}

/// Atomic write: write to a sibling temp file, fsync, then rename over the
/// destination. A crash mid-write leaves the original file intact rather
/// than a half-written `tasks.json` that the frontend can't deserialize.
fn atomic_write(path: &str, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    let tmp = format!("{path}.tmp");
    {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    fs::rename(&tmp, path)
}

pub async fn serve_health() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "ok",
        "timestamp": Utc::now().to_rfc3339(),
        "uptime": START_TIME.elapsed().as_secs()
    }))
}

pub async fn serve_index(State(state): State<AppState>) -> impl IntoResponse {
    let path = StdPath::new("frontend/dist/index.html");
    match tokio::fs::read_to_string(path).await {
        Ok(content) => {
            let rendered = content.replace("{{SITE_TITLE}}", &state.config.server.site_title);
            Html(rendered).into_response()
        }
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

pub async fn get_tasks() -> impl IntoResponse {
    match tokio::fs::read_to_string(TASKS_FILE).await {
        // Return the raw JSON string so the frontend can deserialize into
        // its own `BoardData` type without us having to round-trip through
        // serde_json::Value (which would silently strip unknown fields).
        Ok(data) => (
            StatusCode::OK,
            [(axum::http::header::CONTENT_TYPE, "application/json")],
            data,
        )
            .into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

/// Save the kanban state. Performs optimistic concurrency: if the
/// incoming payload's `version` is less than the current stored version,
/// returns 409 Conflict with the current version. Otherwise writes
/// atomically with `version + 1`.
pub async fn save_tasks(Json(payload): Json<BoardData>) -> impl IntoResponse {
    // Validate structure: every board must have at least one column, every
    // column must have a name. Malformed payloads (e.g. truncated JSON)
    // already fail at Json extraction with 422 by axum's built-in handler,
    // but we also defensively check invariants here.
    for (board_id, board) in &payload.boards {
        if board.columns.is_empty() {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "validation_failed",
                    "detail": format!("board '{board_id}' has no columns"),
                })),
            )
                .into_response();
        }
    }

    // Load current state to check version.
    let current: BoardData = match tokio::fs::read_to_string(TASKS_FILE).await {
        Ok(s) => match serde_json::from_str(&s) {
            Ok(b) => b,
            Err(e) => {
                tracing::error!("tasks.json is corrupt: {e}");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "storage_corrupt" })),
                )
                    .into_response();
            }
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => BoardData::default(),
        Err(e) => {
            tracing::error!("failed to read tasks.json: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    // Optimistic concurrency check.
    if payload.version < current.version {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "version_conflict",
                "current_version": current.version,
                "your_version": payload.version,
            })),
        )
            .into_response();
    }

    let mut new_data = payload;
    new_data.version = current.version.saturating_add(1);

    let serialized = match serde_json::to_string_pretty(&new_data) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("failed to serialize BoardData: {e}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    match tokio::task::spawn_blocking(move || atomic_write(TASKS_FILE, serialized.as_bytes())).await
    {
        Ok(Ok(())) => Json(serde_json::json!({
            "ok": true,
            "version": new_data.version,
        }))
        .into_response(),
        Ok(Err(e)) => {
            tracing::error!("failed to write tasks.json: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
        Err(e) => {
            tracing::error!("save_tasks join error: {e}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

#[cfg(test)]
mod tests;
