use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct DragState {
    start_x: f64,
    start_width: f64,
}

#[derive(PartialEq, Clone, Props)]
pub struct ResizablePanelProps {
    // tailwind classes for the panel itself (background, border, overflow). width is set separately
    #[props(default = String::new())]
    class: String,
    #[props(default = 360.0)]
    default_width: f64,
    #[props(default = 240.0)]
    min_width: f64,
    #[props(default = 800.0)]
    max_width: f64,
    children: Element,
}

// a right-side panel with a drag handle on its left edge, same feel as the Proc/Mess panels. drop
// it where the panel goes in a flex row and it renders the handle, the panel, and (while dragging)
// a full-screen overlay that keeps capturing the mouse even if it leaves the thin handle
#[component]
pub fn ResizablePanel(props: ResizablePanelProps) -> Element {
    let mut width = use_signal(|| props.default_width);
    let mut drag = use_signal(|| None::<DragState>);
    let dragging = drag().is_some();
    let min = props.min_width;
    let max = props.max_width;

    rsx! {
        div {
            // a thin 4px bar (bg-clip-content keeps it slim) padded out into a wider invisible grab zone
            class: "w-1 box-content px-1 shrink-0 bg-gray-700 hover:bg-indigo-500 bg-clip-content cursor-col-resize",
            onmousedown: move |e| {
                drag.set(Some(DragState {
                    start_x: e.client_coordinates().x,
                    start_width: width(),
                }));
            },
        }
        div {
            class: "shrink-0 flex flex-col min-h-0 {props.class}",
            style: "width: {width()}px;",
            {props.children}
        }
        if dragging {
            div {
                class: "fixed inset-0 z-50 cursor-col-resize",
                onmousemove: move |e| {
                    if let Some(state) = drag() {
                        // handle sits on the panel's left, so dragging left (smaller x) widens it
                        let delta = state.start_x - e.client_coordinates().x;
                        width.set((state.start_width + delta).clamp(min, max));
                    }
                },
                onmouseup: move |_| drag.set(None),
                onmouseleave: move |_| drag.set(None),
            }
        }
    }
}
