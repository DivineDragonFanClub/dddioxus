use dioxus::prelude::*;

use super::components_panel::ComponentsPanel;
use super::transform_inspector::TransformInspector;

#[derive(PartialEq, Clone, Props)]
pub struct InspectorProps {
    path: String,
}

#[component]
pub fn Inspector(props: InspectorProps) -> Element {
    rsx! {
        div {
            "data-component": "Inspector",
            class: "flex flex-col shrink-0 bg-gray-900 border-l border-gray-700 overflow-y-auto",
            div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                h3 { class: "text-white font-bold text-sm", "Inspector" }
                p { class: "text-gray-500 text-xs truncate", "{props.path}" }
            }
            TransformInspector { path: props.path.clone() }
            ComponentsPanel { path: props.path }
        }
    }
}
