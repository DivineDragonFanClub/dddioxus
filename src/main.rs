#![allow(non_snake_case)]

extern crate core;

use dioxus::desktop::muda::{Menu, MenuId, MenuItemBuilder};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

use components::globals_view::GlobalsView;
use components::procs_view::ProcsView;
use components::scene_view::SceneView;
use components::shell::Shell;

mod components;
mod hooks;
mod protocol;
mod rpc;

const TAILWIND: Asset = asset!("/assets/tailwind.css");

#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Scene {},
        #[route("/globals")]
        Globals {},
        #[route("/procs")]
        Procs {},
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    let window = WindowBuilder::new().with_title("Divine Debugging Dragon");

    let menu = Menu::new();
    menu.append(
        &MenuItemBuilder::new()
            .text("Schlong")
            .enabled(true)
            .id(MenuId("test".into()))
            .build(),
    )
    .unwrap();

    let config = Config::new().with_menu(menu).with_window(window);

    LaunchBuilder::new().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: TAILWIND }
        document::Style { "html, body {{ margin: 0; height: 100%; overflow: hidden; background: #111827; }}" }
        div { class: "flex flex-col h-screen text-white",
            Router::<Route> {}
        }
    }
}

#[component]
fn Scene() -> Element {
    rsx! { SceneView {} }
}

#[component]
fn Globals() -> Element {
    rsx! { GlobalsView {} }
}

#[component]
fn Procs() -> Element {
    rsx! { ProcsView {} }
}
