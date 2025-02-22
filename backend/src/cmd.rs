use clap::Parser;

/// machine-launcher is the web server to manage the power of servers.
#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Config file
    #[arg(long)]
    pub config: String,
}

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    pub oidc: OidcConfig,
    pub drivers: Vec<DriverType>,
}

#[derive(Debug, Deserialize)]
pub struct OidcConfig {
    // OIDC Issuer Identifier. (https://openid.net/specs/openid-connect-core-1_0.html)
    pub issuer: String,

    // OAuth2.0 Client ID. (https://www.rfc-editor.org/rfc/rfc6749.html)
    pub client_id: String,

    // OAuth2.0 Client Secret. (https://www.rfc-editor.org/rfc/rfc6749.html)
    pub client_secret: String,

    // OAuth2.0 Scope. (https://www.rfc-editor.org/rfc/rfc6749.html)
    #[serde(default = "default_oidc_scopes")]
    pub scopes: String,

    // role_attribute_path is in JMESPath format. Only entities that return true are allowed.
    pub role_attribute_path: String,
}

fn default_oidc_scopes() -> String {
    String::from("openid profile email")
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum DriverType {
    Debug(DriverDebug),
    Ipmi(DriverIpmi),
    Wol(DriverWol),
}

#[derive(Debug, Deserialize)]
pub struct DriverDebug {
    // Name is identifier. It must be unique.
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct DriverIpmi {
    // Name is identifier. It must be unique.
    pub name: String,

    // Server Address to use IPMI, this format must be ADDRESS:PORT
    pub server_addr: String,

    // IPMI username
    pub username: String,

    // IPMI password
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct DriverWol {
    // Name is identifier. It must be unique.
    pub name: String,

    // MAC Address to send magic-packets for Wake-on-LAN
    pub mac_addr: String,

    // IP Address to check the server status for Wake-on-LAN
    pub ip_addr: String,
}
