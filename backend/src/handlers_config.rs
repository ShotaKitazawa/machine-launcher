use std::sync::Arc;

use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};

use machine_launcher_common::{Endpoint, GetOidcConfig, GetUserInfo, OidcConfigResponse, UserInfo};

use crate::{AppState, OidcState};

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(GetOidcConfig::PATH, get(oidc_config))
        .route(GetUserInfo::PATH, get(userinfo))
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
            audience: None,
        }),
        OidcState::Enabled {
            client_id,
            authorization_endpoint,
            token_endpoint,
            audience,
            ..
        } => Json(OidcConfigResponse {
            enabled: true,
            client_id: Some(client_id.clone()),
            authorization_endpoint: Some(authorization_endpoint.clone()),
            token_endpoint: Some(token_endpoint.clone()),
            audience: audience.clone(),
        }),
    }
}

#[cfg_attr(feature = "openapi-gen", utoipa::path(
    get,
    path = "/api/userinfo",
    responses(
        (status = 200, body = UserInfo),
        (status = 401),
    )
))]
pub async fn userinfo(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> Result<Json<UserInfo>, StatusCode> {
    match &state.oidc {
        OidcState::Disabled => Ok(Json(UserInfo {
            sub: "local".to_string(),
            name: Some("Local User".to_string()),
            picture: None,
        })),
        OidcState::Enabled {
            userinfo_endpoint, ..
        } => {
            let auth_header = headers
                .get("authorization")
                .and_then(|v| v.to_str().ok())
                .ok_or(StatusCode::UNAUTHORIZED)?;

            let resp = reqwest::Client::new()
                .get(userinfo_endpoint)
                .header("Authorization", auth_header)
                .send()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

            if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
                return Err(StatusCode::UNAUTHORIZED);
            }
            if !resp.status().is_success() {
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }

            let info: UserInfo = resp
                .json()
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
            Ok(Json(info))
        }
    }
}
