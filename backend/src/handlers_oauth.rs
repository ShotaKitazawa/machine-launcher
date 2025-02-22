use std::collections::HashMap;
use std::sync::Arc;

use axum::{
    extract::{Query, State},
    response::Redirect,
    routing::get,
    Router,
};
use axum_extra::extract::cookie::{Cookie, CookieJar, SameSite};
use openidconnect::core::CoreResponseType;
use openidconnect::{
    AuthenticationFlow, AuthorizationCode, CsrfToken, Nonce, PkceCodeChallenge, Scope,
    TokenResponse,
};

use crate::{AppState, Error};
use machine_launcher_utils::COOKIE_KEY;

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", get(login))
        .route("/logout", get(logout))
        .route("/callback", get(callback))
}

async fn login(State(state): State<Arc<AppState>>) -> Result<Redirect, Error> {
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let (auth_url, csrf_token, _nonce) = state
        .oidc_client
        .authorize_url(
            AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .add_scopes([
            Scope::new("openid".to_string()),
            Scope::new("profile".to_string()),
            Scope::new("email".to_string()),
        ])
        .set_pkce_challenge(pkce_challenge)
        .url();

    state
        .pkce_verifiers
        .lock()
        .unwrap()
        .insert(csrf_token.secret().clone(), pkce_verifier);

    Ok(Redirect::temporary(auth_url.as_str()))
}

async fn logout(cookie_jar: CookieJar) -> Result<(CookieJar, Redirect), Error> {
    // TODO: logout from OIDC Provider using renocation_url
    let mut cookie = Cookie::from(COOKIE_KEY);
    cookie.set_max_age(time::Duration::ZERO);
    cookie.set_path("/");
    Ok((cookie_jar.add(cookie), Redirect::to("/")))
}

async fn callback(
    cookie_jar: CookieJar,
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<(CookieJar, Redirect), Error> {
    let code = params
        .get("code")
        .ok_or_else(|| Error::Unauthorized("Missing authorization code".into()))?;
    let state_param = params
        .get("state")
        .ok_or_else(|| Error::Unauthorized("Missing state parameter".into()))?;
    let pkce_verifier = state
        .pkce_verifiers
        .lock()
        .unwrap()
        .remove(state_param)
        .ok_or_else(|| Error::Unauthorized("Invalid PKCE verifier".into()))?;

    let resp = state
        .oidc_client
        .exchange_code(AuthorizationCode::new(code.to_string()))
        .set_pkce_verifier(pkce_verifier)
        .request_async(&reqwest::Client::new())
        .await
        .map_err(|e| Error::Unauthorized(format!("Token exchange failed: {:?}", e).into()))?;
    let id_token = resp
        .id_token()
        .ok_or_else(|| Error::Unauthorized("Id token is none".into()))?;
    let cookie = Cookie::build((COOKIE_KEY, id_token.to_string()))
        .path("/")
        .http_only(false) // for frontend to refer to JWT's profile
        .same_site(SameSite::Strict)
        .secure(true)
        .build()
        .into_owned();
    Ok((cookie_jar.add(cookie), Redirect::to("/")))
}
