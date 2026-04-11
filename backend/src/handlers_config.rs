use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Local;
use serde::Serialize;

use crate::AppState;

const NONCE_TTL_SECS: i64 = 600; // 10 minutes

#[derive(Debug, Serialize)]
pub struct OidcConfigResponse {
    pub client_id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
}

#[derive(Debug, Serialize)]
pub struct NonceResponse {
    pub nonce: String,
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/oidc-config", get(oidc_config))
        .route("/auth/nonce", get(issue_nonce))
}

async fn oidc_config(State(state): State<Arc<AppState>>) -> Json<OidcConfigResponse> {
    Json(OidcConfigResponse {
        client_id: state.oidc_client_id.clone(),
        authorization_endpoint: state.oidc_authorization_endpoint.clone(),
        token_endpoint: state.oidc_token_endpoint.clone(),
    })
}

async fn issue_nonce(State(state): State<Arc<AppState>>) -> Json<NonceResponse> {
    let bytes: [u8; 32] = rand::random();
    let nonce = URL_SAFE_NO_PAD.encode(bytes);

    let now = Local::now().timestamp();
    let expiry = now + NONCE_TTL_SECS;
    let mut store = state.nonce_store.lock().unwrap();
    store.retain(|_, &mut exp| exp > now); // 期限切れ nonce を掃除
    store.insert(nonce.clone(), expiry);

    Json(NonceResponse { nonce })
}
