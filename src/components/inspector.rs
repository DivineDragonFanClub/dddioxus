use dioxus::prelude::*;

use super::components_panel::ComponentsPanel;
use super::scene_view::RevealRequest;
use super::transform_inspector::TransformInspector;

#[derive(PartialEq, Clone, Props)]
pub struct InspectorProps {
    path: String,
}

// just the inspector contents (header + transform + components). the surrounding resizable column
// is provided by the ResizablePanel that wraps this in scene_view
#[component]
pub fn Inspector(props: InspectorProps) -> Element {
    let reveal = try_consume_context::<RevealRequest>();
    rsx! {
        div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
            h3 { class: "text-white font-bold text-sm", "Inspector" }
            button {
                class: "block max-w-full truncate text-left underline text-indigo-400 hover:text-indigo-300 text-xs cursor-pointer",
                title: "Reveal in scene tree: {props.path}",
                onclick: move |_| {
                    // Force-expand the selected path for this click, then (after the
                    // tree re-renders) scroll to it and flash it.
                    if let Some(RevealRequest(mut nonce)) = reveal {
                        nonce.set(nonce() + 1);
                    }
                    spawn(async move {
                        let _ = document::eval(
                            "(() => { setTimeout(() => { const el = document.querySelector('[data-component=SceneTree] [data-tree-selected=true]'); if (!el) return; el.scrollIntoView({ block: 'center', behavior: 'smooth' }); setTimeout(() => { el.style.transition = 'none'; el.style.backgroundColor = 'rgba(129,140,248,0.85)'; setTimeout(() => { el.style.transition = 'background-color 0.7s ease-out'; el.style.backgroundColor = ''; setTimeout(() => { el.style.transition = ''; }, 750); }, 60); }, 250); }, 150); })()"
                        ).await;
                    });
                },
                "{props.path}"
            }
        }
        TransformInspector { path: props.path.clone() }
        ComponentsPanel { path: props.path.clone() }
    }
}
