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
    // The URL of OpenID Provider. (https://openid.net/specs/openid-connect-core-1_0.html#Terminology)
    pub provider_url: String,

    // OIDC Client Identifer. (https://openid.net/specs/openid-connect-core-1_0.html#Terminology)
    pub client_id: String,

    // OIDC Client Secret. (https://openid.net/specs/openid-connect-core-1_0.html#Terminology)
    pub client_secret: String,

    // role_attribute_path is in JMESPath format. Only entities that return true are allowed.
    pub role_attribute_path: String,
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
