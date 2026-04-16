#![allow(non_snake_case)]

extern crate core;

use dioxus::desktop::muda::{Menu, PredefinedMenuItem, Submenu};
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;
use dioxus_logger::tracing::Level;

use components::connection_provider::ConnectionProvider;
use components::globals_view::GlobalsView;
use components::procs_view::ProcsView;
use components::scene_view::SceneView;
use components::shell::Shell;
use dock::{persistence as dock_persistence, DockState};
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
mod dock;
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
        #[cfg(any(debug_assertions, feature = "dev"))]
        #[route("/dev/scene-simulation")]
        DevSceneSimulation {},
}

fn main() {
    dioxus_logger::init(Level::INFO).expect("failed to init logger");

    let window = WindowBuilder::new().with_title("Divine Debugging Dragon");

    // Build a standard Edit submenu so Cmd+C/V/X/A route through the
    // macOS responder chain to the focused input field. Without these
    // key equivalents registered in the menu bar, the shortcuts never
    // fire inside input elements.
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
    // Establish the connection signal once at the app root so it persists
    // across route changes and is available to any descendant via context.
    use_connection();
    // Dock state (lock/pin/floating/splits) persists across route changes and
    // will become the single source of truth for layout in subsequent phases.
    use_dock_state();
    // Debug/dev-only: start the dioxus-inspector HTTP bridge so Claude
    // Code (and other MCP-aware tooling) can read the DOM, run JS, etc.
    use_inspector_bridge();

    rsx! {
        document::Stylesheet { href: TAILWIND }
        document::Style { "html, body {{ margin: 0; height: 100%; overflow: hidden; background: #111827; }}" }
        div { class: "flex flex-col h-screen text-white",
            Router::<Route> {}
        }
    }
}

/// Load the persisted dock state (or fall back to the default layout),
/// provide it via context, and spawn a debounced autosaver. Any change
/// to `DockState` queues a 250 ms write to `~/.dddioxus_layout.json`;
/// rapid mutations coalesce into a single disk write.
fn use_dock_state() -> Signal<DockState> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Arc;

    let initial = dock_persistence::load().unwrap_or_else(DockState::default_layout);
    let state = use_signal(|| initial);
    use_hook(|| provide_context(state));

    // Shared monotonic counter. Each effect run bumps it; the async timer
    // captures its own snapshot and only saves if still current on wake.
    let version: Arc<AtomicU64> = use_hook(|| Arc::new(AtomicU64::new(0)));

    use_effect(move || {
        // Subscribe to any field of the dock state.
        let _ = state.read();
        let my_version = version.fetch_add(1, Ordering::SeqCst) + 1;
        let version = version.clone();
        spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            if version.load(Ordering::SeqCst) != my_version {
                return;
            }
            let snapshot = state.peek().clone();
            tokio::task::spawn_blocking(move || dock_persistence::save(&snapshot));
        });
    });

    state
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
    rsx! {
        ConnectionProvider {
            SceneView {}
        }
    }
}

#[component]
fn Globals() -> Element {
    rsx! {
        ConnectionProvider {
            GlobalsView {}
        }
    }
}

#[component]
fn Procs() -> Element {
    rsx! {
        ConnectionProvider {
            ProcsView {}
        }
    }
}
