use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use gloo::storage::{LocalStorage, SessionStorage, Storage};
use gloo::utils::window;
use gloo_timers::callback::Interval;
use js_sys::Date;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use machine_launcher_utils::all_claims_from_jwt;
use openapi::apis::app_api::list_servers;
use openapi::apis::configuration::Configuration;

mod components;
use components::contents::Contents;
use components::footer::Footer;
use components::header::Header;

mod state;
use state::{Server, Userinfo};

const TOKEN_KEY: &str = "id_token";
const PKCE_VERIFIER_KEY: &str = "pkce_verifier";
const OAUTH_STATE_KEY: &str = "oauth_state";
const NONCE_KEY: &str = "nonce";

#[derive(Debug, Clone, Deserialize)]
pub struct OidcConfig {
    pub client_id: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    id_token: String,
}

fn generate_pkce_verifier() -> String {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).unwrap();
    URL_SAFE_NO_PAD.encode(bytes)
}

fn generate_pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

fn generate_state() -> String {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes).unwrap();
    URL_SAFE_NO_PAD.encode(bytes)
}

pub async fn fetch_oidc_config() -> Option<OidcConfig> {
    let url = format!("{}/api/oidc-config", window().origin());
    let resp = reqwest::get(&url).await.ok()?;
    resp.json::<OidcConfig>().await.ok()
}

pub async fn fetch_nonce() -> Option<String> {
    #[derive(Deserialize)]
    struct NonceResponse {
        nonce: String,
    }
    let url = format!("{}/api/auth/nonce", window().origin());
    let resp = reqwest::get(&url).await.ok()?;
    resp.json::<NonceResponse>().await.ok().map(|r| r.nonce)
}

pub async fn start_login() {
    let Some(config) = fetch_oidc_config().await else {
        gloo::console::error!("Failed to fetch OIDC config");
        return;
    };
    let Some(nonce) = fetch_nonce().await else {
        gloo::console::error!("Failed to fetch nonce");
        return;
    };

    let verifier = generate_pkce_verifier();
    let challenge = generate_pkce_challenge(&verifier);
    let state = generate_state();

    SessionStorage::set(PKCE_VERIFIER_KEY, &verifier).ok();
    SessionStorage::set(OAUTH_STATE_KEY, &state).ok();
    SessionStorage::set(NONCE_KEY, &nonce).ok();

    let redirect_uri = format!("{}/callback", window().origin());
    let auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope=openid+profile+email\
         &code_challenge={}&code_challenge_method=S256&state={}&nonce={}",
        config.authorization_endpoint,
        config.client_id,
        js_sys::encode_uri_component(&redirect_uri),
        challenge,
        state,
        nonce,
    );

    window().location().set_href(&auth_url).ok();
}

pub async fn logout() {
    LocalStorage::delete(TOKEN_KEY);
    window().location().set_href("/").ok();
}

async fn handle_callback() -> Option<String> {
    let search = window().location().search().ok()?;
    let params = parse_query_string(&search);

    let code = params.get("code")?.clone();
    let state_param = params.get("state")?.clone();

    let stored_state = SessionStorage::get::<String>(OAUTH_STATE_KEY).ok()?;
    if state_param != stored_state {
        gloo::console::error!("OAuth state mismatch");
        return None;
    }

    let verifier = SessionStorage::get::<String>(PKCE_VERIFIER_KEY).ok()?;
    SessionStorage::delete(PKCE_VERIFIER_KEY);
    SessionStorage::delete(OAUTH_STATE_KEY);
    SessionStorage::delete(NONCE_KEY);

    let config = fetch_oidc_config().await?;
    let redirect_uri = format!("{}/callback", window().origin());

    let body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        js_sys::encode_uri_component(&code),
        js_sys::encode_uri_component(&redirect_uri),
        js_sys::encode_uri_component(&config.client_id),
        js_sys::encode_uri_component(&verifier),
    );

    let client = reqwest::Client::new();
    let resp = client
        .post(&config.token_endpoint)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .ok()?;

    let token_resp: TokenResponse = resp.json().await.ok()?;
    LocalStorage::set(TOKEN_KEY, &token_resp.id_token).ok();

    window().location().set_href("/").ok();
    Some(token_resp.id_token)
}

fn parse_query_string(search: &str) -> std::collections::HashMap<String, String> {
    let s = search.trim_start_matches('?');
    s.split('&')
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?.to_string();
            let val = parts.next().unwrap_or("").to_string();
            Some((key, val))
        })
        .collect()
}

#[function_component]
fn App() -> Html {
    // States
    let user = use_state(|| None as Option<Userinfo>);
    let servers = use_state(|| vec![] as Vec<Server>);
    let token = use_state(|| None as Option<String>);

    // On mount: handle /callback or load token from localStorage
    {
        let user = user.clone();
        let token = token.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                // Check if we're in the OAuth callback
                let search = window().location().search().unwrap_or_default();
                if search.contains("code=") {
                    handle_callback().await;
                    return;
                }

                // Load existing token from localStorage
                if let Ok(t) = LocalStorage::get::<String>(TOKEN_KEY) {
                    if let Ok(claims) = all_claims_from_jwt(&t) {
                        let expired_at = claims
                            .get("exp")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        if get_unix_time() > expired_at {
                            LocalStorage::delete(TOKEN_KEY);
                            user.set(None);
                            token.set(None);
                        } else {
                            user.set(Some(Userinfo {
                                name: claims
                                    .get("name")
                                    .map_or_else(|| "".to_string(), |v| v.to_string()),
                                icon_url: claims
                                    .get("picture")
                                    .map_or_else(|| "".to_string(), |v| v.to_string()),
                            }));
                            token.set(Some(t));
                        }
                    }
                }
            });
            || ()
        });
    };

    // Poll server list every 10 seconds
    {
        let servers = servers.clone();
        let user = user.clone();
        let token = token.clone();
        use_effect(move || {
            let f = move || {
                if user.is_none() {
                    return;
                }
                let current_token = (*token).clone();
                let servers = servers.clone();
                let user = user.clone();
                let mut c = Configuration::new();
                c.base_path = window().origin();
                c.bearer_access_token = current_token;
                spawn_local(async move {
                    match list_servers(&c).await {
                        Ok(res) => {
                            if !compare_servers(res.clone(), servers.to_vec()) {
                                servers.set(res)
                            }
                        }
                        Err(e) => {
                            gloo::console::log!(format!("{:?}", e));
                            // TODO: display error message in browser
                            user.set(None)
                        }
                    }
                });
            };
            f();
            let handle = Interval::new(10000, f);
            move || drop(handle) // cleanup
        });
    };

    html! {
        <div>
            <section class="machine-launcher">
                <Header user={(*user).clone()} />
                <div class="fixed w-screen flex justify-center ">
                    <Contents
                        user={(*user).clone()}
                        servers={(*servers).clone()}
                    />
                </div>
                <Footer />
            </section>
        </div>
    }
}

fn compare_servers(mut a: Vec<Server>, mut b: Vec<Server>) -> bool {
    a.sort_by(|x, y| x.hostname.cmp(&y.hostname));
    b.sort_by(|x, y| x.hostname.cmp(&y.hostname));
    a == b
}

fn get_unix_time() -> f64 {
    Date::now() / 1000.0
}

fn main() {
    yew::Renderer::<App>::new().render();
}
