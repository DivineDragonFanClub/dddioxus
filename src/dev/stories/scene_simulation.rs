use dioxus::prelude::*;

use crate::components::components_panel::ComponentsListPanel;
use crate::components::scene_view::ScenePanel;
use crate::components::transform_inspector::TransformPanel;
use crate::dev::fixtures;

/// Full-layout "simulation" of the Scene viewer driven entirely by
/// fixtures — no live game connection required. Mirrors the real
/// SceneView layout (ScenePanel + Inspector column) but swaps in
/// TransformPanel / ComponentsListPanel with canned data where the
/// real app uses TransformInspector / ComponentsPanel (which fetch).
///
/// Clicking nodes in the tree drives the Inspector's header path.
/// The fixture data in the Inspector stays the same regardless of
/// which node is selected — it's a layout demo, not a true mock.
#[component]
pub fn DevSceneSimulation() -> Element {
    let mut selected_path = use_signal(|| None::<String>);
    let toggle_noop = use_callback(|_: u32| {});

    rsx! {
        div { class: "flex flex-1 h-full",
            ScenePanel {
                data: Some(Ok(fixtures::scene_loaded())),
                loading: false,
                selected_path: selected_path(),
                on_refresh: |_| {},
                on_select: move |path: String| selected_path.set(Some(path)),
                on_toggle_active: |_: String| {},
            }
            match selected_path() {
                Some(path) => rsx! {
                    div { class: "flex flex-col shrink-0 bg-gray-900 border-l border-gray-700 overflow-y-auto",
                        div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                            h3 { class: "text-white font-bold text-sm", "Inspector" }
                            p { class: "text-gray-500 text-xs truncate", "{path}" }
                        }
                        TransformPanel {
                            data: Some(Ok(fixtures::transform_nonzero())),
                            on_refresh: |_| {},
                            on_change: |_| {},
                        }
                        ComponentsListPanel {
                            data: Some(Ok(fixtures::components_loaded())),
                            on_refresh: |_| {},
                            on_toggle: toggle_noop,
                        }
                    }
                },
                None => rsx! {
                    div { class: "flex flex-col shrink-0 bg-gray-900 border-l border-gray-700 overflow-y-auto",
                        div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                            h3 { class: "text-white font-bold text-sm", "Inspector" }
                            p { class: "text-gray-500 text-xs", "No selection" }
                        }
                        p { class: "p-4 text-gray-500 text-xs italic",
                            "Select a node in the scene tree to inspect its transform and components."
                        }
                    }
                },
            }
        }
    }
}
