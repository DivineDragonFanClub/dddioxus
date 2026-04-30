use dioxus::prelude::*;

use super::components_panel::ComponentsPanel;
use super::transform_inspector::TransformInspector;

/// Inline style pinning the Inspector column's width. Pinned so the
/// column doesn't reflow as its contents go through "no selection" →
/// "loading…" → filled-with-RPC-data states (each of which would
/// otherwise size to its own natural content width and flash).
///
/// Uses an inline `style` instead of a Tailwind class because this
/// project ships a precompiled Tailwind v2 bundle (no JIT for
/// arbitrary `w-[...]` values).
pub const INSPECTOR_WIDTH_STYLE: &str = "width: 440px;";

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
            style: "{INSPECTOR_WIDTH_STYLE}",
            div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                h3 { class: "text-white font-bold text-sm", "Inspector" }
                p { class: "text-gray-500 text-xs truncate", "{props.path}" }
            }
            TransformInspector { path: props.path.clone() }
            ComponentsPanel { path: props.path }
        }
    }
}
