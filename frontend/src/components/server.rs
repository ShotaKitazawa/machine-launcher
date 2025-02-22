use gloo::utils::window;
use wasm_bindgen_futures::spawn_local;
use yew::prelude::*;

use openapi::apis::app_api::{start_server, stop_server};
use openapi::apis::configuration::Configuration;
use openapi::models::ServerName;

#[derive(PartialEq, Properties)]
pub struct ServerProps {
    pub server: crate::state::Server,
}

#[function_component]
pub fn Server(props: &ServerProps) -> Html {
    let is_open = use_state(|| false);
    let server_status_color = if props.server.running {
        "#27C940"
    } else {
        "#FF5F59"
    };

    html! {
        <div class="basis-1/6 w-full max-y-sm max-w-sm bg-white border border-gray-200 rounded-lg shadow-sm dark:bg-gray-800 dark:border-gray-700 mx-4">
            <div class="flex justify-end px-4 pt-4">
                <svg class="w-5 h-5" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 100 100">
                    <path fill={server_status_color} d="m50,0 a50,50,0,0,0,0,100 a-50,-50,0,0,0,0,-100" />
                </svg>
            </div>

            <div class="flex flex-col items-center">
                <img class="w-24 h-24 mb-3 rounded-full shadow-lg" src="/public/server.png" alt="Server image"/>
                <h5 class="p-4 mb-1 text-xl font-medium text-gray-900 dark:text-white">{props.server.name.clone()}</h5>
                <span class="text-sm text-gray-500 dark:text-gray-400">{props.server.hostname.clone()}</span>
                <div class="flex my-4 md:mt-6">
                    <ServerDialog
                        server_name={props.server.name.clone()}
                        is_running={props.server.running}
                        is_open={is_open.clone()}
                    />
                </div>
            </div>
        </div>
    }
}

#[derive(PartialEq, Properties)]
pub struct ServerDialogProps {
    pub server_name: String,
    pub is_running: bool,
    pub is_open: UseStateHandle<bool>,
}

#[function_component]
pub fn ServerDialog(props: &ServerDialogProps) -> Html {
    let on_open = {
        let is_open = props.is_open.clone();
        Callback::from(move |_: MouseEvent| is_open.set(true))
    };

    html! {
        <div>
            <button
                class="block text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:outline-none focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 text-center dark:bg-blue-600 dark:hover:bg-blue-700 dark:focus:ring-blue-800"
                type="button"
                onclick={on_open}
            >
            {if props.is_running {"Stop Machine"} else {"Start Machine"} }
            </button>
            <ServerModal
                server_name={props.server_name.clone()}
                is_running={props.is_running}
                is_open={props.is_open.clone()}
            />
        </div>
    }
}

#[function_component]
pub fn ServerModal(props: &ServerDialogProps) -> Html {
    let on_cancel = {
        let is_open = props.is_open.clone();
        Callback::from(move |_: MouseEvent| is_open.set(false))
    };
    let on_switch_server = {
        let server_name = props.server_name.clone();
        let is_running = props.is_running;
        let is_open = props.is_open.clone();
        Callback::from(move |_: MouseEvent| {
            let server_name = server_name.clone();
            let mut c = Configuration::new();
            c.base_path = window().origin();
            spawn_local(async move {
                match is_running {
                    true => {
                        if let Err(e) = stop_server(&c, ServerName { name: server_name }).await {
                            gloo::console::log!(format!("{:?}", e))
                        }
                    }
                    false => {
                        if let Err(e) = start_server(&c, ServerName { name: server_name }).await {
                            gloo::console::log!(format!("{:?}", e))
                        }
                    }
                }
            });
            is_open.set(false)
        })
    };

    let dialog_icon_svgpath = if props.is_running {
        "M10 11V6m0 8h.01M19 10a9 9 0 1 1-18 0 9 9 0 0 1 18 0Z"
    } else {
        "M4 10L8 14L16 6"
    };
    let dialog_message = if props.is_running {
        "Are you sure you want to stop this machine?"
    } else {
        "Are you sure you want to start this machine?"
    };
    let dialog_button_color = if props.is_running {
        "bg-red-600 hover:bg-red-800 focus:ring-red-300 dark:focus:ring-red-800"
    } else {
        "bg-green-600 hover:bg-green-800 focus:ring-green-300 dark:focus:ring-green-800"
    };

    if *props.is_open {
        html! {
        <div tabindex="-1" class="fixed inset-0 flex overflow-y-auto overflow-x-hidden z-50 justify-center items-center w-full h-[calc(100%-1rem)] max-h-full">
                <div class="z-50 p-4 w-full max-w-md max-h-full">
                    <div class="relative bg-white rounded-lg shadow-sm dark:bg-gray-700">
                        <div class="p-4 md:p-5 text-center">
                            <svg class="mx-auto mb-4 text-gray-400 w-12 h-12 dark:text-gray-200" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
                                <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d={dialog_icon_svgpath}/>
                            </svg>
                            <h3 class="mb-5 text-lg font-normal text-gray-500 dark:text-gray-400">{dialog_message}</h3>
                            <button
                                class={format!("{} text-white focus:ring-4 focus:outline-none font-medium rounded-lg text-sm inline-flex items-center px-5 py-2.5 text-center", dialog_button_color)}
                                type="button"
                                onclick={on_switch_server.clone()}
                            >
                                {"Yes, I'm sure"}
                            </button>
                            <button
                                class="py-2.5 px-5 ms-3 text-sm font-medium text-gray-900 focus:outline-none bg-white rounded-lg border border-gray-200 hover:bg-gray-100 hover:text-blue-700 focus:z-10 focus:ring-4 focus:ring-gray-100 dark:focus:ring-gray-700 dark:bg-gray-800 dark:text-gray-400 dark:border-gray-600 dark:hover:text-white dark:hover:bg-gray-700"
                                type="button"
                                onclick={on_cancel.clone()}
                            >
                                {"No, cancel"}
                            </button>
                        </div>
                    </div>
                </div>
            <div
              class="fixed bg-black bg-opacity-50 w-full h-full z-10"
              onclick={on_cancel.clone()}
            ></div>
        </div>
        }
    } else {
        html! {}
    }
}
