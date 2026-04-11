use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::{collections::HashSet, str::FromStr};

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
use jmespath::Variable;
use chrono::Local;
use openidconnect::{
    core::{CoreIdToken, CoreIdTokenClaims},
    Nonce, NonceVerifier,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, Error};
use machine_launcher_utils::all_claims_from_jwt;

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    _permissions: Option<HashSet<String>>,
}

struct StoredNonceVerifier {
    nonce_store: Arc<Mutex<HashMap<String, i64>>>,
}
impl NonceVerifier for &StoredNonceVerifier {
    fn verify(self, nonce: Option<&Nonce>) -> Result<(), String> {
        let nonce_val = nonce.ok_or("nonce claim is missing from token")?;
        let nonce_str = nonce_val.secret();
        let mut store = self
            .nonce_store
            .lock()
            .map_err(|_| "nonce store lock error".to_string())?;
        let expiry = store
            .remove(nonce_str)
            .ok_or("nonce not found or already used")?;
        if Local::now().timestamp() > expiry {
            return Err("nonce has expired".to_string());
        }
        Ok(())
    }
}

pub async fn auth_middleware(
    State(state): State<Arc<AppState>>,
    mut req: Request,
    next: Next,
) -> Result<impl IntoResponse, Error> {
    // get credential from Authorization: Bearer header
    let id_token_str = req
        .extract_parts::<TypedHeader<Authorization<Bearer>>>()
        .await
        .map_err(|_| Error::Unauthorized("You are not logged in, please provide token".into()))?
        .token()
        .to_string();

    let id_token: CoreIdToken = openidconnect::IdToken::from_str(&id_token_str)
        .map_err(|e| Error::Forbidden(format!("Provided token is not IdToken: {:?}", e).into()))?;
    let nonce_verifier = StoredNonceVerifier {
        nonce_store: state.nonce_store.clone(),
    };
    let _: &CoreIdTokenClaims = id_token
        .claims(
            &state.oidc_client.id_token_verifier(),
            &nonce_verifier,
        )
        .map_err(|e| Error::Forbidden(format!("Provided token is invalid: {:?}", e).into()))?;

    // MEMO: CoreIdToken cannot contain custom claims decided dynamically,
    // so get all claims as HashMap after JWT validations.
    let all_claims =
        all_claims_from_jwt(&id_token_str).map_err(|e| Error::Forbidden(e.to_string().into()))?;

    // And, authorized based on JMESPath
    authorize_based_jmespath(all_claims, state)?;

    Ok(next.run(req).await)
}

fn authorize_based_jmespath<T: Serialize>(claims: T, state: Arc<AppState>) -> Result<(), Error> {
    let jmespath_data = Variable::from_serializable(claims)
        .map_err(|e| Error::InternalServerError(format!("invalid claims: {:?}", e).into()))?;
    if !state
        .role_attribute_path_expr
        .search(jmespath_data)
        // JMESPath evaluation was not matched
        .map_err(|_| Error::Forbidden("no role found".into()))?
        .as_boolean()
        // result of JMESPath is not boolean
        .ok_or_else(|| Error::Forbidden("no role found".into()))?
    {
        // result of JMESPath is false
        Err(Error::Forbidden("no role found".into()))
    } else {
        Ok(())
    }
}
