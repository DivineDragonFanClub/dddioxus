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

// Urls are relative to your Cargo.toml file
// const _TAILWIND_URL: &str = manganis::mg!(file("assets/main.css"));


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
        .with_window(window);

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
        WebSocketProvider {
            MessageContextProvider {
                Message { }
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
        Link {
            to: Route::Blog {
                id: 69
            },
            "Go to blog"
        }
        div {
            div { Message { } }
        }
    }
}
