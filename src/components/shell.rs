use dioxus::prelude::*;

use crate::dock::{self, DockCommand, DockState, PanelKind};
use crate::Route;

#[component]
pub fn Shell() -> Element {
    let current: Route = use_route();

    #[cfg(any(debug_assertions, feature = "dev"))]
    let on_dev = crate::dev::is_dev_route(&current);
    #[cfg(not(any(debug_assertions, feature = "dev")))]
    let on_dev = false;
    #[cfg(not(any(debug_assertions, feature = "dev")))]
    let _ = &current;

    rsx! {
        div {
            "data-component": "Shell",
            class: "flex flex-1 overflow-hidden",
            if on_dev {
                {dev_sidebar()}
            } else {
                Sidebar {}
            }
            div { class: "flex-1 overflow-hidden",
                Outlet::<Route> {}
            }
        }
    }
}

#[cfg(any(debug_assertions, feature = "dev"))]
fn dev_sidebar() -> Element {
    rsx! { crate::dev::DevSidebar {} }
}

#[cfg(not(any(debug_assertions, feature = "dev")))]
fn dev_sidebar() -> Element {
    rsx! {}
}

#[component]
fn Sidebar() -> Element {
    let mut state = use_context::<Signal<DockState>>();

    let open_panel = move |kind: PanelKind| {
        dock::apply(&mut state.write(), DockCommand::OpenPanel { kind });
    };
    let reset_layout = move |_| {
        dock::apply(&mut state.write(), DockCommand::ResetLayout);
    };

    rsx! {
        nav { class: "w-40 shrink-0 bg-gray-950 border-r border-gray-700 flex flex-col py-2",
            PanelButton {
                label: "Scene",
                kind: PanelKind::Scene,
                on_open: open_panel,
            }
            PanelButton {
                label: "Globals",
                kind: PanelKind::Globals,
                on_open: open_panel,
            }
            PanelButton {
                label: "Procs",
                kind: PanelKind::Procs,
                on_open: open_panel,
            }
            PanelButton {
                label: "Inspector",
                kind: PanelKind::Inspector,
                on_open: open_panel,
            }
            div { class: "mt-auto pt-2 border-t border-gray-800",
                button {
                    class: "block w-full text-left px-4 py-2 text-xs text-gray-500 hover:text-white hover:bg-gray-800",
                    title: "Reset layout to default",
                    onclick: reset_layout,
                    "Reset layout"
                }
                {dev_nav_item()}
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct PanelButtonProps {
    label: &'static str,
    kind: PanelKind,
    on_open: EventHandler<PanelKind>,
}

#[component]
fn PanelButton(props: PanelButtonProps) -> Element {
    let state = use_context::<Signal<DockState>>();
    let present = state
        .read()
        .bindings
        .values()
        .any(|b| b.kind == props.kind);
    let kind = props.kind;
    let on_open = props.on_open;
    let class = if present {
        "block w-full text-left px-4 py-2 text-sm text-white bg-gray-800 hover:bg-gray-700"
    } else {
        "block w-full text-left px-4 py-2 text-sm text-gray-300 hover:bg-gray-800"
    };
    let title = if present {
        format!("Focus {} panel", props.label)
    } else {
        format!("Open {} panel", props.label)
    };

    rsx! {
        button {
            class: "{class}",
            title: "{title}",
            onclick: move |_| on_open.call(kind),
            "{props.label}"
        }
    }
}

#[cfg(any(debug_assertions, feature = "dev"))]
fn dev_nav_item() -> Element {
    rsx! {
        Link {
            to: Route::DevIndex {},
            class: "block px-4 py-2 text-sm text-gray-300 hover:bg-gray-800",
            "UI Storybook"
        }
    }
}

#[cfg(not(any(debug_assertions, feature = "dev")))]
fn dev_nav_item() -> Element {
    rsx! {}
}
