use dioxus::prelude::*;
use dioxus_elements::input_data::MouseButton;

use super::model::{Axis, DockState};
use super::path::{self, DockPath};
use crate::dock::DockNode;

#[derive(Clone, Debug, PartialEq)]
struct DragState {
    split: DockPath,
    axis: Axis,
    /// Bounding rect of the Split's container in CSS pixels: (x, y, w, h).
    rect: (f64, f64, f64, f64),
}

#[derive(PartialEq, Clone, Props)]
pub struct SplitterProps {
    pub path: DockPath,
    pub axis: Axis,
}

/// 3px drag-handle that sets its parent Split's ratio based on cursor
/// position inside the Split's bounding rect. Uses a viewport-wide overlay
/// during drag so releases / leaves outside the handle still land.
#[component]
pub fn Splitter(props: SplitterProps) -> Element {
    let mut state = use_context::<Signal<DockState>>();
    let mut drag = use_signal(|| None::<DragState>);
    let axis = props.axis;
    let horizontal = matches!(axis, Axis::Horizontal);
    let path = props.path.clone();

    let start_drag = {
        let path = path.clone();
        move |evt: MouseEvent| {
            evt.prevent_default();
            // Walk from the cursor up to the nearest `data-component="Split"`
            // element and return its rect. We can't rely on the Splitter's
            // own rect because we need the whole Split's extent.
            let point = evt.client_coordinates();
            let mut drag = drag;
            let path = path.clone();
            spawn(async move {
                let script = format!(
                    "var p = document.elementFromPoint({px}, {py});
                     while (p && p.getAttribute('data-component') !== 'Split') p = p.parentElement;
                     if (!p) {{ dioxus.send(null); return; }}
                     var r = p.getBoundingClientRect();
                     dioxus.send([r.left, r.top, r.width, r.height]);",
                    px = point.x,
                    py = point.y
                );
                let mut eval = document::eval(&script);
                if let Ok(val) = eval.recv::<serde_json::Value>().await {
                    if let Some(arr) = val.as_array() {
                        if arr.len() == 4 {
                            let rect = (
                                arr[0].as_f64().unwrap_or(0.0),
                                arr[1].as_f64().unwrap_or(0.0),
                                arr[2].as_f64().unwrap_or(0.0),
                                arr[3].as_f64().unwrap_or(0.0),
                            );
                            drag.set(Some(DragState {
                                split: path,
                                axis,
                                rect,
                            }));
                        }
                    }
                }
            });
        }
    };

    let drag_now = drag();

    rsx! {
        div {
            "data-component": "Splitter",
            class: if horizontal {
                "w-[3px] bg-gray-800 hover:bg-indigo-500 transition-colors shrink-0"
            } else {
                "h-[3px] bg-gray-800 hover:bg-indigo-500 transition-colors shrink-0"
            },
            style: if horizontal { "cursor: ew-resize" } else { "cursor: ns-resize" },
            onmousedown: start_drag,
        }
        if let Some(d) = drag_now {
            div {
                "data-component": "SplitterOverlay",
                class: "fixed inset-0 z-50",
                style: if horizontal { "cursor: ew-resize" } else { "cursor: ns-resize" },
                onmousemove: {
                    let d = d.clone();
                    move |e: MouseEvent| {
                        if !e.held_buttons().contains(MouseButton::Primary) {
                            drag.set(None);
                            return;
                        }
                        let coord = e.client_coordinates();
                        let (rx, ry, rw, rh) = d.rect;
                        let ratio = if horizontal {
                            ((coord.x - rx) / rw) as f32
                        } else {
                            ((coord.y - ry) / rh) as f32
                        };
                        let ratio = ratio.clamp(0.05, 0.95);
                        let mut w = state.write();
                        if let Some(DockNode::Split { ratio: r, .. }) =
                            path::node_at_mut(&mut w.main_tree, &d.split)
                        {
                            *r = ratio;
                        }
                    }
                },
                onmouseup: move |_| drag.set(None),
            }
        }
    }
}
