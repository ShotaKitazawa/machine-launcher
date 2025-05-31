use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

use axum::{routing::get_service, serve, Router};
use clap::Parser;
use jmespath::compile;
use machine_launcher::drivers::ipmi::IpmiDriver;
use once_cell::sync::Lazy;
use openidconnect::{
    core::CoreProviderMetadata, ClientId, ClientSecret, IssuerUrl, RedirectUrl, TokenUrl,
};
use regex::Regex;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
use url::Url;

use machine_launcher::{
    cmd::{Args, Config, DriverType},
    drivers::{debug::DebugDriver, traits::PowerManagerTrait, wake_on_lan::WakeOnLanDriver},
    AppState, OidcClient,
};

const COMPILED_FILES_PATH: &str = "../frontend/dist/";
const STATIC_FILES_PATH: &str = "../frontend/public/";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup logger
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "info,tower_http=debug,axum::rejection=trace");
    }
    tracing_subscriber::fmt::init();

    // Parse arguments
    let args = Args::parse();

    // Parse config file
    let config_str =
        std::fs::read_to_string(args.config).expect("args.config should be readable as file");
    let config_str_templated =
        substitute_env_variables(&config_str).expect("config should be templated by envsubst");
    let config: Config = toml::from_str(&config_str_templated).expect("config should be parsed");

    // Drivers
    let mut drivers = HashMap::<String, Arc<dyn PowerManagerTrait>>::new();
    for driver_conf in config.drivers {
        match driver_conf {
            DriverType::Debug(c) => {
                drivers.insert(c.name.clone(), Arc::new(DebugDriver::new(c.name.clone())))
            }
            DriverType::Ipmi(c) => drivers.insert(
                c.name.clone(),
                Arc::new(
                    IpmiDriver::new(c.name.clone(), c.server_addr, c.username, c.password)
                        .expect("IpmiDriver cloud not be initialized"),
                ),
            ),
            DriverType::Wol(c) => drivers.insert(
                c.name.clone(),
                Arc::new(
                    WakeOnLanDriver::new(c.name.clone(), c.mac_addr, c.ip_addr)
                        .expect("WakeOnLanDriver cloud not be initialized"),
                ),
            ),
        };
    }

    // OIDC Client
    let http_client = &reqwest::Client::new();
    let oidc_provider_metadata = CoreProviderMetadata::discover_async(
        IssuerUrl::new(config.oidc.provider_url)?,
        http_client,
    )
    .await
    .expect("Failed to discover provider metadata");
    let oidc_client = OidcClient::from_provider_metadata(
        oidc_provider_metadata.clone(),
        ClientId::new(config.oidc.client_id),
        Some(ClientSecret::new(config.oidc.client_secret)),
    )
    .set_redirect_uri(RedirectUrl::new(
        Url::parse(&config.url)
            .unwrap()
            .join("/auth/callback")
            .unwrap()
            .to_string(),
    )?)
    // TODO: use set_revocation_url
    //.set_revocation_url(RevocationUrl::new(
    //    oidc_provider_metadata
    //        .additional_metadata()
    //        .revocation_endpoint
    //        .clone(),
    //)?)
    .set_token_uri(TokenUrl::new(
        oidc_provider_metadata
            .clone()
            .token_endpoint()
            .unwrap()
            .to_string(),
    )?);
    let pkce_verifiers = Mutex::new(HashMap::new());

    // Authorization based on ID Token
    let role_attribute_path_expr = compile(&config.oidc.role_attribute_path)
        .expect("args.role_attribute_path should be JMESPath format.");

    // AppState
    let app_state = Arc::new(AppState {
        drivers,
        oidc_client,
        role_attribute_path_expr,
        pkce_verifiers,
    });

    // Routing
    let app = Router::new()
        .nest(
            "/api",
            machine_launcher::handlers_app::routes(app_state.clone()),
        )
        .nest("/auth", machine_launcher::handlers_oauth::routes())
        .nest_service("/public", get_service(ServeDir::new(STATIC_FILES_PATH)))
        .fallback_service(get_service(ServeDir::new(COMPILED_FILES_PATH)))
        .with_state(app_state)
        .layer(TraceLayer::new_for_http());

    // Serve
    let tcp_listener =
        if let Some(std_listener) = listenfd::ListenFd::from_env().take_tcp_listener(0)? {
            TcpListener::from_std(std_listener)?
        } else {
            TcpListener::bind(&SocketAddr::from(([0, 0, 0, 0], 8080))).await?
        };
    serve(tcp_listener, app).await?;

    Ok(())
}

static RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"\$\{(\w+)\}").unwrap());

fn substitute_env_variables(input: &str) -> Result<String, Box<dyn std::error::Error>> {
    if envsubst::is_templated(input) {
        let mut context = std::collections::HashMap::new();
        for (_, [cap]) in RE.captures_iter(input).map(|c| c.extract()) {
            let val = env::var(cap)?;
            context.insert(cap.to_string(), val);
        }
        let res = envsubst::substitute(input, &context)?;
        Ok(res)
    } else {
        Ok(input.to_string())
    }
}
