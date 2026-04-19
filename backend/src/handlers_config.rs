use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::Local;

use machine_launcher_common::{
    Endpoint, GetNonce, GetOidcConfig, NonceResponse, OidcConfigResponse,
};

use crate::AppState;

const NONCE_TTL_SECS: i64 = 600;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(GetOidcConfig::PATH, get(oidc_config))
        .route(GetNonce::PATH, get(issue_nonce))
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    get,
    path = "/api/oidc-config",
    responses((status = 200, body = OidcConfigResponse))
))]
async fn oidc_config(State(state): State<Arc<AppState>>) -> Json<OidcConfigResponse> {
    Json(OidcConfigResponse {
        client_id: state.oidc_client_id.clone(),
        authorization_endpoint: state.oidc_authorization_endpoint.clone(),
        token_endpoint: state.oidc_token_endpoint.clone(),
    })
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    get,
    path = "/api/auth/nonce",
    responses((status = 200, body = NonceResponse))
))]
async fn issue_nonce(State(state): State<Arc<AppState>>) -> Json<NonceResponse> {
    let bytes: [u8; 32] = rand::random();
    let nonce = URL_SAFE_NO_PAD.encode(bytes);

    let now = Local::now().timestamp();
    let expiry = now + NONCE_TTL_SECS;
    let mut store = state.nonce_store.lock().unwrap();
    store.retain(|_, &mut exp| exp > now);
    store.insert(nonce.clone(), expiry);

    Json(NonceResponse { nonce })
}
