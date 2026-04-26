use std::collections::HashSet;
use std::sync::Arc;

use axum::RequestExt;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::IntoResponse,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{AppState, Error, OidcState};

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    if matches!(state.oidc, OidcState::Disabled) {
        return Ok(next.run(req).await);
    }

    let OidcState::Enabled {
        userinfo_endpoint,
        allowed_subs,
        ..
    } = &state.oidc
    else {
        unreachable!()
    };

    let mut req = req;
    let token_str = req
        .extract_parts::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| Error::Unauthorized("You are not logged in, please provide token".into()))?
        .token()
        .to_string();

    // Validate token by proxying to the IdP's userinfo endpoint
    let resp = reqwest::Client::new()
        .get(userinfo_endpoint)
        .bearer_auth(&token_str)
        .send()
        .await
        .map_err(|_| {
            Error::InternalServerError("Failed to contact IdP userinfo endpoint".into())
        })?;

    if resp.status() == reqwest::StatusCode::UNAUTHORIZED {
        return Err(Error::Unauthorized("Token rejected by IdP".into()));
    }
    if !resp.status().is_success() {
        return Err(Error::InternalServerError(
            "IdP userinfo returned unexpected status".into(),
        ));
    }

    let userinfo: serde_json::Value = resp
        .json()
        .await
        .map_err(|_| Error::InternalServerError("Failed to parse IdP userinfo response".into()))?;

    let sub = userinfo
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Forbidden("sub claim missing from userinfo response".into()))?;

    authorize_by_sub(sub, allowed_subs)?;

    Ok(next.run(req).await)
}

fn authorize_by_sub(sub: &str, allowed_subs: &HashSet<String>) -> Result<(), Error> {
    if allowed_subs.is_empty() {
        return Ok(());
    }
    if allowed_subs.contains(sub) {
        Ok(())
    } else {
        Err(Error::Forbidden("sub not in allowed list".into()))
    }
}
