use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};
use shared_backend::auth::{is_locked_out, record_attempt, reset_attempts};
use shared_backend::server::get_client_ip;
use std::net::SocketAddr;
use std::time::Duration;

use super::COOKIE_NAME;
use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct VerifyPinPayload {
    pub pin: Option<String>,
}

pub async fn verify_pin(
    headers: HeaderMap,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    Json(payload): Json<VerifyPinPayload>,
) -> impl IntoResponse {
    let pin_req = &state.config.server.pin;
    if pin_req.is_none() {
        return (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response();
    }

    let ip = get_client_ip(
        &headers,
        addr,
        state.config.server.trust_proxy,
        &state.config.server.trusted_proxies,
    );

    let ip_str = ip.to_string();
    let lockout_dur = Duration::from_secs(state.config.server.lockout_time_minutes * 60);
    if is_locked_out(&ip_str, state.config.server.max_attempts, lockout_dur) {
        let secs_remaining = shared_backend::auth::lockout_remaining_secs(&ip_str, lockout_dur);
        let minutes_remaining = (secs_remaining / 60).max(1);
        return (
            StatusCode::TOO_MANY_REQUESTS,
            Json(serde_json::json!({
                "success": false,
                "error": format!("Too many attempts. Please try again in {} minute(s).", minutes_remaining)
            })),
        )
            .into_response();
    }

    let expected_pin = match pin_req.as_ref() {
        Some(p) => p,
        None => {
            return (StatusCode::OK, Json(serde_json::json!({ "success": true }))).into_response();
        }
    };
    let pin_str = payload.pin.as_deref().unwrap_or("").trim();

    if pin_str.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "success": false, "error": "PIN is required." })),
        )
            .into_response();
    }

    if constant_time_eq::constant_time_eq(pin_str.as_bytes(), expected_pin.as_bytes()) {
        reset_attempts(&ip_str);

        let session_id = crate::session_id::generate_session_id();
        state
            .active_sessions
            .write()
            .await
            .insert(session_id.clone());

        let secure = crate::cookie_auth::cookie_should_be_secure(
            &headers,
            &state.config.server.base_url,
        );

        let cookie = crate::cookie_auth::build_cookie(&session_id,
            state.config.server.cookie_max_age_hours,
            secure,
        );
        let cookie_str = cookie.to_string();
        let mut headers = HeaderMap::new();
        if let Ok(val) = header::HeaderValue::from_str(&cookie_str) {
            headers.insert(header::SET_COOKIE, val);
        }
        (
            StatusCode::OK,
            headers,
            Json(serde_json::json!({ "success": true })),
        )
            .into_response()
    } else {
        let attempt = record_attempt(&ip_str);
        let remaining = (state.config.server.max_attempts as i64 - attempt.count as i64).max(0);

        (
            StatusCode::UNAUTHORIZED,
            Json(serde_json::json!({
                "success": false,
                "error": "Invalid PIN",
                "attemptsLeft": remaining
            })),
        )
            .into_response()
    }
}
