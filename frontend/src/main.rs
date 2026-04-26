use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use gloo::storage::{LocalStorage, SessionStorage, Storage};
use gloo::utils::window;
use gloo_timers::callback::Interval;
use machine_launcher_common::Endpoint;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

mod components;
use components::contents::Contents;
use components::footer::Footer;
use components::header::Header;

mod state;
use state::{Server, Userinfo};

const ACCESS_TOKEN_KEY: &str = "access_token";
const PKCE_VERIFIER_KEY: &str = "pkce_verifier";
const OAUTH_STATE_KEY: &str = "oauth_state";
const NONCE_KEY: &str = "nonce";

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
}

fn generate_random_bytes_b64(len: usize) -> String {
    let mut bytes = vec![0u8; len];
    getrandom::fill(&mut bytes).unwrap();
    URL_SAFE_NO_PAD.encode(&bytes)
}

fn generate_pkce_challenge(verifier: &str) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()))
}

pub async fn fetch_oidc_config() -> Option<machine_launcher_common::OidcConfigResponse> {
    reqwest::Client::new()
        .get(format!(
            "{}{}",
            window().origin(),
            machine_launcher_common::GetOidcConfig::PATH
        ))
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()
}

async fn fetch_userinfo(token: &str) -> Option<machine_launcher_common::UserInfo> {
    reqwest::Client::new()
        .get(format!(
            "{}{}",
            window().origin(),
            machine_launcher_common::GetUserInfo::PATH
        ))
        .bearer_auth(token)
        .send()
        .await
        .ok()?
        .json()
        .await
        .ok()
}

pub async fn start_login() {
    let Some(config) = fetch_oidc_config().await else {
        gloo::console::error!("Failed to fetch OIDC config");
        return;
    };
    if !config.enabled {
        return;
    }

    let verifier = generate_random_bytes_b64(32);
    let challenge = generate_pkce_challenge(&verifier);
    let state = generate_random_bytes_b64(16);
    let nonce = generate_random_bytes_b64(16);

    SessionStorage::set(PKCE_VERIFIER_KEY, &verifier).ok();
    SessionStorage::set(OAUTH_STATE_KEY, &state).ok();
    SessionStorage::set(NONCE_KEY, &nonce).ok();

    let redirect_uri = format!("{}/callback", window().origin());
    let mut auth_url = format!(
        "{}?response_type=code&client_id={}&redirect_uri={}&scope=openid+profile+email\
         &code_challenge={}&code_challenge_method=S256&state={}&nonce={}",
        config.authorization_endpoint.unwrap_or_default(),
        config.client_id.unwrap_or_default(),
        js_sys::encode_uri_component(&redirect_uri),
        challenge,
        state,
        nonce,
    );
    if let Some(aud) = config.audience {
        auth_url.push_str(&format!("&audience={}", js_sys::encode_uri_component(&aud)));
    }

    window().location().set_href(&auth_url).ok();
}

pub async fn logout() {
    LocalStorage::delete(ACCESS_TOKEN_KEY);
    window().location().set_href("/").ok();
}

async fn handle_callback() {
    let search = window().location().search().unwrap_or_default();
    let params = parse_query_string(&search);

    let Some(code) = params.get("code").cloned() else {
        return;
    };
    let Some(state_param) = params.get("state").cloned() else {
        return;
    };

    let Ok(stored_state) = SessionStorage::get::<String>(OAUTH_STATE_KEY) else {
        return;
    };
    if state_param != stored_state {
        gloo::console::error!("OAuth state mismatch");
        return;
    }

    let Ok(verifier) = SessionStorage::get::<String>(PKCE_VERIFIER_KEY) else {
        return;
    };
    SessionStorage::delete(PKCE_VERIFIER_KEY);
    SessionStorage::delete(OAUTH_STATE_KEY);
    SessionStorage::delete(NONCE_KEY);

    let Some(config) = fetch_oidc_config().await else {
        return;
    };
    let redirect_uri = format!("{}/callback", window().origin());

    let body = format!(
        "grant_type=authorization_code&code={}&redirect_uri={}&client_id={}&code_verifier={}",
        js_sys::encode_uri_component(&code),
        js_sys::encode_uri_component(&redirect_uri),
        js_sys::encode_uri_component(&config.client_id.unwrap_or_default()),
        js_sys::encode_uri_component(&verifier),
    );

    let http_client = reqwest::Client::new();
    let Ok(resp) = http_client
        .post(&config.token_endpoint.unwrap_or_default())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
    else {
        return;
    };

    let Ok(token_resp) = resp.json::<TokenResponse>().await else {
        return;
    };
    LocalStorage::set(ACCESS_TOKEN_KEY, &token_resp.access_token).ok();
    window().location().set_href("/").ok();
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
    let user = use_state(|| None as Option<Userinfo>);
    let servers = use_state(|| vec![] as Vec<Server>);
    let token = use_state(|| None as Option<String>);
    let oidc_enabled = use_state(|| None as Option<bool>);

    {
        let user = user.clone();
        let token = token.clone();
        let oidc_enabled = oidc_enabled.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                // Handle OIDC callback
                let search = window().location().search().unwrap_or_default();
                if search.contains("code=") {
                    handle_callback().await;
                    return;
                }

                // Check OIDC config
                let Some(config) = fetch_oidc_config().await else {
                    return;
                };
                oidc_enabled.set(Some(config.enabled));

                // Dev mode: skip auth entirely
                if !config.enabled {
                    user.set(Some(Userinfo {
                        name: "Local Dev".to_string(),
                        icon_url: String::new(),
                    }));
                    return;
                }

                // Check stored access token via /api/userinfo
                let Ok(stored_token) = LocalStorage::get::<String>(ACCESS_TOKEN_KEY) else {
                    return;
                };
                let Some(info) = fetch_userinfo(&stored_token).await else {
                    LocalStorage::delete(ACCESS_TOKEN_KEY);
                    return;
                };
                user.set(Some(Userinfo {
                    name: info.name.unwrap_or_default(),
                    icon_url: info.picture.unwrap_or_default(),
                }));
                token.set(Some(stored_token));
            });
            || ()
        });
    };

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
                spawn_local(async move {
                    let mut req = reqwest::Client::new().get(format!(
                        "{}{}",
                        window().origin(),
                        machine_launcher_common::ListServers::PATH
                    ));
                    if let Some(t) = current_token {
                        req = req.bearer_auth(t);
                    }
                    let result = async { req.send().await?.json::<Vec<Server>>().await }.await;
                    match result {
                        Ok(res) => {
                            if !compare_servers(res.clone(), servers.to_vec()) {
                                servers.set(res)
                            }
                        }
                        Err(e) => {
                            gloo::console::log!(format!("{:?}", e));
                            user.set(None)
                        }
                    }
                });
            };
            f();
            let handle = Interval::new(10000, f);
            move || drop(handle)
        });
    };

    html! {
        <div>
            <section class="machine-launcher">
                <Header user={(*user).clone()} oidc_enabled={*oidc_enabled} />
                <div class="fixed w-screen flex justify-center ">
                    <Contents
                        user={(*user).clone()}
                        servers={(*servers).clone()}
                        oidc_enabled={*oidc_enabled}
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

fn main() {
    yew::Renderer::<App>::new().render();
}
