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

#[cfg(any(debug_assertions, feature = "dev"))]
use dev::{
    DevComponentRow, DevComponentsListPanel, DevDescsPanel, DevGlobalRow, DevGlobalsPanel,
    DevIndex, DevProcTreeNode, DevProcsPanel, DevScenePanel, DevSceneTree, DevTransformPanel,
    DevVec3Editor,
};

mod components;
#[cfg(any(debug_assertions, feature = "dev"))]
pub mod dev;
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

        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev")]
        DevIndex {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/vec3-editor")]
        DevVec3Editor {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/scene-tree")]
        DevSceneTree {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/scene-panel")]
        DevScenePanel {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/global-row")]
        DevGlobalRow {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/globals-panel")]
        DevGlobalsPanel {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/component-row")]
        DevComponentRow {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/components-list-panel")]
        DevComponentsListPanel {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/transform-panel")]
        DevTransformPanel {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/proc-tree-node")]
        DevProcTreeNode {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/descs-panel")]
        DevDescsPanel {},
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/procs-panel")]
        DevProcsPanel {},
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
