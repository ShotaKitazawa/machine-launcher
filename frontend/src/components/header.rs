use yew::prelude::*;

#[derive(PartialEq, Properties)]
pub struct HeaderProps {
    pub user: Option<crate::state::Userinfo>,
}

#[function_component]
pub fn Header(props: &HeaderProps) -> Html {
    html! {
    <div class="pb-10">
        <header class="bg-[#4169e1] text-white p-1 flex justify-between items-center">
            <div class="flex items-center">
                <img src="/public/icon.png" alt="Logo" width={64} height={64} />
                <h2>{"machine launcher"}</h2>
            </div>
            <HeaderLogin user={props.user.clone()} />
        </header>
    </div>
    }
}

#[function_component]
pub fn HeaderLogin(props: &HeaderProps) -> Html {
    let is_open = use_state(|| false);
    let hidden_class = if *is_open { "" } else { "hidden" };
    let icon_url = if props.user.is_some() {
        let url = props.user.clone().unwrap().icon_url;
        url[1..url.len() - 1].to_string()
    } else {
        String::from("/public/default-avator.svg")
    };
    let usermenu_toggle = {
        let is_open = is_open.clone();
        move |_| is_open.set(!*is_open)
    };

    if props.user.is_none() {
        html! {
            <div class="relative pr-4">
                <button onclick={usermenu_toggle} class="flex items-center focus:outline-none">
                    <a href="/auth/login">{"LOGIN"}</a>
                </button>
            </div>
        }
    } else {
        html! {
            <div class="relative pr-4">
                <button onclick={usermenu_toggle} class="flex items-center focus:outline-none">
                    <img
                        src={ icon_url }
                        alt="User avatar"
                        width={32}
                        height={32}
                        class="rounded-full"
                    />
                </button>
                <div class={classes!(
                    "absolute", "right-0", "mt-2", "w-48", "bg-white",
                    "rounded-md", "shadow-lg", "py-1", "z-10", hidden_class,
                )}>
                <a href="/auth/logout">
                    <button class="block px-4 py-2 text-sm text-gray-700 hover:bg-gray-100 w-full text-left inline-block mr-2" size={16} >
                        {"Logout"}
                    </button>
                </a>
                </div>
            </div>

        }
    }
}
