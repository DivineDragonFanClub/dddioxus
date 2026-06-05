#![allow(non_snake_case)]

extern crate core;

use dioxus::desktop::muda::{Menu, PredefinedMenuItem, Submenu};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

use components::bonds_view::BondsView;
use components::catalog_provider::CatalogProvider;
use components::connection_provider::ConnectionProvider;
use components::cutscene_view::CutsceneView;
use components::forces::ForceView;
use components::map_view::MapView;
use components::globals_view::GlobalsView;
use components::mess_view::MessView;
use components::procs_view::ProcsView;
use components::scene_view::SceneView;
use components::script_view::ScriptView;
use components::shell::Shell;
use hooks::connection::use_connection;

#[cfg(any(debug_assertions, feature = "dev"))]
use dev::{
    DevComponentRow, DevComponentsListPanel, DevDescsPanel, DevGlobalRow, DevGlobalsPanel,
    DevIndex, DevProcTreeNode, DevProcsPanel, DevScenePanel, DevSceneSimulation, DevSceneTree,
    DevTransformPanel, DevVec3Editor,
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
        #[route("/map")]
        Map {},
        #[route("/forces")]
        Forces {},
        #[route("/bonds")]
        Bonds {},
        #[route("/globals")]
        Globals {},
        #[route("/procs")]
        Procs {},
        #[route("/mess")]
        Mess {},
        #[route("/script")]
        Script {},
        #[route("/cutscene")]
        Cutscene {},

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
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/scene-simulation")]
        DevSceneSimulation {},
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    let window = WindowBuilder::new().with_title("Divine Debugging Dragon");

    let menu = Menu::new();
    let edit_menu = Submenu::new("Edit", true);
    edit_menu
        .append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ])
        .unwrap();
    menu.append(&edit_menu).unwrap();

    let config = Config::new().with_menu(menu).with_window(window);

    LaunchBuilder::new().with_cfg(config).launch(App);
}

#[component]
fn App() -> Element {
    use_connection();
    // Debug/dev-only: start the dioxus-inspector HTTP bridge so Claude
    // Code (and other MCP-aware tooling) can read the DOM, run JS, etc.
    use_inspector_bridge();

    rsx! {
        document::Stylesheet { href: TAILWIND }
        document::Style { "html, body {{ margin: 0; height: 100%; overflow: hidden; background: #111827; }}" }
        div { class: "flex flex-col h-screen text-white",
            ConnectionProvider {
                CatalogProvider {
                    Router::<Route> {}
                }
            }
        }
    }
}

/// Start the dioxus-inspector HTTP bridge on port 9999. Only active
/// in debug or `dev`-feature builds; otherwise a no-op.
#[cfg(any(debug_assertions, feature = "dev"))]
fn use_inspector_bridge() {
    use dioxus_inspector::{start_bridge, EvalResponse};
    use_hook(|| {
        let mut eval_rx = start_bridge(9999, "dddioxus");
        spawn(async move {
            while let Some(cmd) = eval_rx.recv().await {
                let response = match document::eval(&cmd.script).await {
                    Ok(val) => EvalResponse::success(val.to_string()),
                    Err(e) => EvalResponse::error(e.to_string()),
                };
                let _ = cmd.response_tx.send(response);
            }
        });
    });
}

#[cfg(not(any(debug_assertions, feature = "dev")))]
fn use_inspector_bridge() {}

#[component]
fn Scene() -> Element {
    rsx! { SceneView {} }
}

#[component]
fn Map() -> Element {
    rsx! { MapView {} }
}

#[component]
fn Forces() -> Element {
    rsx! { ForceView {} }
}

#[component]
fn Bonds() -> Element {
    rsx! { BondsView {} }
}

#[component]
fn Globals() -> Element {
    rsx! { GlobalsView { temporary_only: false } }
}

#[component]
fn Procs() -> Element {
    rsx! { ProcsView {} }
}

#[component]
fn Mess() -> Element {
    rsx! { MessView {} }
}

#[component]
fn Script() -> Element {
    rsx! { ScriptView {} }
}

#[component]
fn Cutscene() -> Element {
    rsx! { CutsceneView {} }
}
