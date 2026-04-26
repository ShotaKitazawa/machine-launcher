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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[cfg_attr(feature = "utoipa", derive(utoipa::ToSchema))]
pub struct UserInfo {
    pub sub: String,
    pub name: Option<String>,
    pub picture: Option<String>,
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

pub struct GetUserInfo;
impl Endpoint for GetUserInfo {
    type Request = ();
    type Response = UserInfo;
    const PATH: &'static str = "/api/userinfo";
    const METHOD: &'static str = "GET";
}
