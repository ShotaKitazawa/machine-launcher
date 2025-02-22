use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use base64::prelude::*;
use serde_json::Value;

pub const COOKIE_KEY: &str = "token";

pub fn all_claims_from_jwt(jwt: &str) -> Result<HashMap<String, Value>, Error> {
    let token_payload = String::from_utf8(
        BASE64_STANDARD_NO_PAD
            .decode(jwt.split('.').collect::<Vec<&str>>()[1])
            .map_err(|e| {
                Error::new(
                    ErrorKind::InvalidInput,
                    format!("failed to decode JWT from base64: {}", e),
                )
            })?,
    )
    .map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("failed to cast JWT from Vec<u8> to String: {}", e),
        )
    })?;
    let mut all_claims = HashMap::new();
    let id_token_json: Value = serde_json::from_str(&token_payload).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("Provided token is not IdToken: {:?}", e),
        )
    })?;
    if let Value::Object(map) = id_token_json {
        for (key, value) in map {
            all_claims.insert(key.clone(), value.clone());
        }
    };
    Ok(all_claims)
}
