use std::collections::HashSet;
use std::str::FromStr;
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
use openidconnect::{core::CoreIdToken, Nonce, NonceVerifier};

use crate::{AppState, Error, OidcState};
use machine_launcher_common::all_claims_from_jwt;

struct SkipNonceVerifier;
impl NonceVerifier for SkipNonceVerifier {
    fn verify(self, _nonce: Option<&Nonce>) -> Result<(), String> {
        Ok(())
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    if matches!(state.oidc, OidcState::Disabled) {
        return Ok(next.run(req).await);
    }

    let OidcState::Enabled {
        client,
        allowed_subs,
        ..
    } = &state.oidc
    else {
        unreachable!()
    };

    let mut req = req;
    let id_token_str = req
        .extract_parts::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| Error::Unauthorized("You are not logged in, please provide token".into()))?
        .token()
        .to_string();

    let id_token: CoreIdToken = openidconnect::IdToken::from_str(&id_token_str)
        .map_err(|e| Error::Forbidden(format!("Provided token is not IdToken: {:?}", e).into()))?;

    id_token
        .claims(&client.id_token_verifier(), SkipNonceVerifier)
        .map_err(|e| Error::Forbidden(format!("Provided token is invalid: {:?}", e).into()))?;

    let all_claims =
        all_claims_from_jwt(&id_token_str).map_err(|e| Error::Forbidden(e.to_string().into()))?;

    authorize_by_sub(all_claims, allowed_subs)?;

    Ok(next.run(req).await)
}

fn authorize_by_sub(
    claims: std::collections::HashMap<String, serde_json::Value>,
    allowed_subs: &HashSet<String>,
) -> Result<(), Error> {
    if allowed_subs.is_empty() {
        return Ok(());
    }
    let sub = claims
        .get("sub")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Forbidden("sub claim missing from token".into()))?;
    if allowed_subs.contains(sub) {
        Ok(())
    } else {
        Err(Error::Forbidden("sub not in allowed list".into()))
    }
}
