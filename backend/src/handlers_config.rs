use std::sync::Arc;

use axum::{extract::State, routing::get, Json, Router};

use machine_launcher_common::{Endpoint, GetOidcConfig, OidcConfigResponse};

use crate::{AppState, OidcState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new().route(GetOidcConfig::PATH, get(oidc_config))
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    get,
    path = "/api/oidc-config",
    responses((status = 200, body = OidcConfigResponse))
))]
pub async fn oidc_config(State(state): State<Arc<AppState>>) -> Json<OidcConfigResponse> {
    match &state.oidc {
        OidcState::Disabled => Json(OidcConfigResponse {
            enabled: false,
            client_id: None,
            authorization_endpoint: None,
            token_endpoint: None,
        }),
        OidcState::Enabled {
            client_id,
            authorization_endpoint,
            token_endpoint,
            ..
        } => Json(OidcConfigResponse {
            enabled: true,
            client_id: Some(client_id.clone()),
            authorization_endpoint: Some(authorization_endpoint.clone()),
            token_endpoint: Some(token_endpoint.clone()),
        }),
    }
}
