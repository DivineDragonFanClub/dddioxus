#![allow(non_snake_case)]

extern crate core;

use dioxus::desktop::{Config, use_wry_event_handler, WindowBuilder};
use dioxus::desktop::muda::{Menu, MenuEvent, MenuId, MenuItemBuilder, PredefinedMenuItem};
use dioxus::desktop::muda::MenuItemKind::MenuItem;
use dioxus::prelude::*;
use dioxus::desktop::tao::event::Event as WryEvent;
use dioxus::desktop::tao::event::WindowEvent;
use log::LevelFilter;

use hooks::use_command;
use protocol::GetSceneNameRequest;
use components::websocketprovider::WebSocketProvider;
use components::messagecontextprovider::MessageContextProvider;
use crate::protocol::{GetProcTreeRequest, GetSceneNameResponse};

mod protocol;
mod components;
mod hooks;

#[derive(Clone, Routable, Debug, PartialEq)]
enum Route {
    #[route("/")]
    Home {},
    #[route("/blog/:id")]
    Blog { id: i32 },
}

fn main() {
    // Init debug
    dioxus_logger::init(LevelFilter::Info).expect("failed to init logger");

    let window = WindowBuilder::new()
        .with_title("Divine Debugging Dragon");

    let menu = Menu::new();
    menu.append(&MenuItemBuilder::new().text("Schlong").enabled(true).id(MenuId("test".into())).build()).unwrap();

    let config = Config::new()
        .with_menu(menu)
        .with_window(window)
        .with_custom_head(r#"<script src="https://cdn.tailwindcss.com"></script>"#.to_string());


    LaunchBuilder::new()
        .with_cfg(config)
        .launch(App);
}

#[component]
fn App() -> Element {
    // Example of event listener for the window

    // use_wry_event_handler(move |event, _| {
    //     if let WryEvent::WindowEvent {
    //         event: WindowEvent::Focused(new_focused),
    //         ..
    //     } = event
    //     {
    //         // focused.set(*new_focused)
    //     }
    // });

    rsx! {
        Router::<Route> {

        }
    }
}

#[component]
fn Testing() -> Element {
    rsx! {
        section {
            class: "grid place-items-center bg-emerald-800 p-16 min-h-screen",
            label {
                input {
                    class:"peer/showLabel absolute scale-0",
                    r#type:"checkbox",
                }
                span {
                    class:"block max-h-14 max-w-xs overflow-hidden rounded-lg bg-emerald-100 px-4 py-0 text-cyan-800 shadow-lg transition-all duration-300 peer-checked/showLabel:max-h-52",
                    h3 {
                        class:"flex h-14 cursor-pointer items-center font-bold",
                        "Expand & Collapse Me",
                    }
                    p {
                        class:"mb-2",
                        "You've crafted a sleek collapsible panel using Tailwind CSS without the need for JavaScript. Impressive! 😎",
                    }
                }
            }
        }
    }
}

#[component]
fn Blog(id: i32) -> Element {
    rsx! {
        Link { to: Route::Home {}, "Go to counter" }
        "Blog post {id}"
    }
}

#[component]
fn Message() -> Element {
    let msg = GetSceneNameRequest;

    // Send the message and get the response back as a Resource.
    let mut res = use_command(msg);


    let suspend_proc = move |_| {
        spawn(async move {
            use_command(GetProcTreeRequest);
        });
    };

    match &*res.read_unchecked() {
        Some(Some(resp)) => {
            rsx! {
                div {
                    "Response: {resp.scene_name}"
                }
                button { class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg",
                    onclick: move |_| res.restart(),
                    { "Send again" }
                }
                button { class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg",
                    onclick: suspend_proc,
                    { "Print processes" }
                }
            }
        },
        None => {
            rsx! { div { "show a spinner idk" } }
        },
        Some(None) => rsx! {
            div {
                "Message response could not be acquired"
            }
            button { class: "text-white bg-indigo-500 border-0 py-2 px-6 focus:outline-none hover:bg-indigo-600 rounded text-lg",
                    onclick: move |_| needs_update(),
                    { "Send again" }
                }
        }
    }
}

#[component]
fn Home() -> Element {
    rsx! {
        WebSocketProvider {
            MessageContextProvider {
                div {
                    div { Message { } }
                }
            }
        }
    }
}
