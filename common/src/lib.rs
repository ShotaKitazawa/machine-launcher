use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use base64::prelude::*;
use serde_json::Value;


pub fn all_claims_from_jwt(jwt: &str) -> Result<HashMap<String, Value>, Error> {
    let token_payload = String::from_utf8(
        BASE64_URL_SAFE_NO_PAD
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

// --- API schema types ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct Server {
    pub name: String,
    pub hostname: String,
    pub running: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ServerName {
    pub name: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct ErrorMessage {
    pub error: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct OidcConfigResponse {
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_endpoint: Option<String>,
}

// --- Endpoint trait ---

pub trait Endpoint {
    type Request: serde::Serialize;
    type Response: serde::de::DeserializeOwned;
    const PATH: &'static str;
    const METHOD: &'static str;
}

pub struct ListServers;
impl Endpoint for ListServers {
    type Request = ();
    type Response = Vec<Server>;
    const PATH: &'static str = "/api/servers";
    const METHOD: &'static str = "GET";
}

pub struct StartServer;
impl Endpoint for StartServer {
    type Request = ServerName;
    type Response = Server;
    const PATH: &'static str = "/api/servers/start";
    const METHOD: &'static str = "PUT";
}

pub struct StopServer;
impl Endpoint for StopServer {
    type Request = ServerName;
    type Response = Server;
    const PATH: &'static str = "/api/servers/stop";
    const METHOD: &'static str = "PUT";
}

pub struct GetOidcConfig;
impl Endpoint for GetOidcConfig {
    type Request = ();
    type Response = OidcConfigResponse;
    const PATH: &'static str = "/api/oidc-config";
    const METHOD: &'static str = "GET";
}
