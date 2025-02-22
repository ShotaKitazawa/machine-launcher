use gloo::utils::window;
use gloo_timers::callback::Interval;
use js_sys::Date;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use machine_launcher_utils::{all_claims_from_jwt, COOKIE_KEY};
use openapi::apis::app_api::list_servers;
use openapi::apis::configuration::Configuration;

mod components;
use components::contents::Contents;
use components::footer::Footer;
use components::header::Header;

mod state;
use state::{Server, Userinfo};

#[function_component]
fn App() -> Html {
    // States
    let user = use_state(|| None as Option<Userinfo>);
    let servers = use_state(|| vec![] as Vec<Server>);

    // Effects
    {
        let user = user.clone();
        use_effect(move || match wasm_cookies::all().unwrap().get(COOKIE_KEY) {
            Some(token) => {
                if user.is_none() {
                    if let Ok(claims) = all_claims_from_jwt(token) {
                        let expired_at = claims.get("exp").unwrap().as_f64().unwrap();
                        let now = get_unix_time();
                        if now > expired_at {
                            // TODO: display error message in browser
                            wasm_cookies::delete(COOKIE_KEY);
                            user.set(None);
                        } else {
                            user.set(Some(Userinfo {
                                name: claims
                                    .get("name")
                                    .map_or_else(|| "".to_string(), |v| v.to_string()),
                                icon_url: claims
                                    .get("picture")
                                    .map_or_else(|| "".to_string(), |v| v.to_string()),
                            }))
                        }
                    }
                }
            }
            None => {
                if user.is_some() {
                    user.set(None)
                }
            }
        });
    };
    {
        let servers = servers.clone();
        let user = user.clone();
        use_effect(move || {
            let f = move || {
                if user.is_none() {
                    return;
                }
                let servers = servers.clone();
                let user = user.clone();
                let mut c = Configuration::new();
                c.base_path = window().origin();
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
