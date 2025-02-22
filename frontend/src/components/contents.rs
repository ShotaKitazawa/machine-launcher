use yew::prelude::*;

use super::server::Server as ServerItem;

#[derive(PartialEq, Properties)]
pub struct ContentsProps {
    pub user: Option<crate::state::Userinfo>,
    pub servers: Vec<crate::state::Server>,
}

#[function_component]
pub fn Contents(props: &ContentsProps) -> Html {
    let hidden_class = if props.servers.is_empty() {
        "hidden"
    } else {
        ""
    };

    if props.user.is_none() {
        html! {
        <div>
            <a href="/auth/login">
                <button type="button" class="text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center inline-flex items-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800">
                    {"Please login with OIDC Provider"}
                    <svg class="rtl:rotate-180 w-3.5 h-3.5 ms-2" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 14 10">
                        <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M1 5h12m0 0L9 1m4 4L9 9"/>
                    </svg>
                </button>
            </a>
        </div>
            }
    } else {
        html! {
            <div class={classes!("main", "flex", "justify-center", "w-full", hidden_class)}>
                { for props.servers.iter().map(|server| {
                html! { <ServerItem server = {server.clone()} /> }
                })}
            </div>
        }
    }
}
