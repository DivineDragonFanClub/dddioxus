use dioxus::prelude::*;

use super::components_panel::ComponentsPanel;
use super::transform_inspector::TransformInspector;
use crate::dock::{selectors, Binding, DockState};

#[component]
pub fn InspectorHost() -> Element {
    let state = use_context::<Signal<DockState>>();
    let bindings = selectors::inspector_bindings(&state.read());

    rsx! {
        div {
            "data-component": "InspectorHost",
            class: "flex flex-col shrink-0 bg-gray-900 border-l border-gray-700 overflow-y-auto",
            if bindings.is_empty() {
                div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                    h3 { class: "text-white font-bold text-sm", "Inspector" }
                    p { class: "text-gray-500 text-xs", "No inspectors" }
                }
            } else {
                for b in bindings {
                    InspectorFrame { key: "{b.id.0}", binding: b.clone() }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct InspectorFrameProps {
    pub binding: Binding,
}

#[component]
pub fn InspectorFrame(props: InspectorFrameProps) -> Element {
    let mut state = use_context::<Signal<DockState>>();
    let binding = props.binding.clone();
    let id = binding.id;
    let follows = binding.follows_selection;
    let path = binding.path.clone();
    let label = path.clone().unwrap_or_else(|| "(no selection)".to_string());

    let toggle_follow = use_callback(move |_: ()| {
        let mut w = state.write();
        let already = w.bindings.get(&id).map(|b| b.follows_selection).unwrap_or(false);
        if already {
            if let Some(b) = w.bindings.get_mut(&id) {
                b.follows_selection = false;
            }
        } else {
            selectors::set_follow_flag_exclusive(&mut w, id);
        }
    });

    let pin = use_callback(move |_: ()| {
        selectors::pin_follow_inspector(&mut state.write());
    });

    let close = use_callback(move |_: ()| {
        selectors::remove_binding(&mut state.write(), id);
    });

    rsx! {
        div {
            "data-component": "InspectorFrame",
            "data-binding": "{id.0}",
            class: "flex flex-col border-b border-gray-700",
            div { class: "flex items-center gap-1 px-3 py-2 bg-gray-800",
                div { class: "flex-1 min-w-0",
                    h3 { class: "text-white font-bold text-xs uppercase tracking-wide",
                        if follows { "Inspector · following" } else { "Inspector · locked" }
                    }
                    p { class: "text-gray-500 text-xs truncate", "{label}" }
                }
                button {
                    class: "text-gray-400 hover:text-white text-sm px-1",
                    title: if follows { "Click to stop following" } else { "Click to follow selection" },
                    onclick: move |_| toggle_follow.call(()),
                    if follows { "🔓" } else { "🔒" }
                }
                if follows {
                    button {
                        class: "text-gray-400 hover:text-white text-sm px-1",
                        title: "Pin current path to a new locked inspector",
                        onclick: move |_| pin.call(()),
                        "📌"
                    }
                } else {
                    button {
                        class: "text-gray-400 hover:text-red-400 text-sm px-1",
                        title: "Close this inspector",
                        onclick: move |_| close.call(()),
                        "×"
                    }
                }
            }
            match path {
                Some(p) => rsx! {
                    TransformInspector { path: p.clone() }
                    ComponentsPanel { path: p }
                },
                None => rsx! {
                    p { class: "p-4 text-gray-500 text-xs italic",
                        "Select a node in the scene tree to bind this inspector."
                    }
                },
            }
        }
    }
}

