use dioxus::prelude::*;

#[derive(Clone, PartialEq, Debug)]
struct DragState {
    start_x: f64,
    start_width: f64,
}

// which edge the drag handle sits on. Right = handle on the panel's left edge, the panel is the
// right-hand pane (scene/map inspectors). Left = handle on the panel's right edge, the panel is the
// left-hand pane (mess/procs lists)
#[derive(PartialEq, Clone, Copy, Default)]
pub enum Side {
    #[default]
    Right,
    Left,
}

#[derive(PartialEq, Clone, Props)]
pub struct ResizablePanelProps {
    // tailwind classes for the panel itself (background, border, overflow). width is set separately
    #[props(default = String::new())]
    class: String,
    #[props(default)]
    side: Side,
    #[props(default = 360.0)]
    default_width: f64,
    #[props(default = 240.0)]
    min_width: f64,
    #[props(default = 800.0)]
    max_width: f64,
    children: Element,
}

// a panel with a thin drag handle on one edge. drop it in a flex row and it renders the handle, the
// panel, and (while dragging) a full screen overlay that keeps capturing the mouse even when it
// leaves the thin handle. the same handle look/feel everywhere, whichever side it lives on
#[component]
pub fn ResizablePanel(props: ResizablePanelProps) -> Element {
    let mut width = use_signal(|| props.default_width);
    let mut drag = use_signal(|| None::<DragState>);
    let dragging = drag().is_some();
    let min = props.min_width;
    let max = props.max_width;
    let side = props.side;

    // a 2px colored bar inside a slightly wider transparent grab zone (bg-clip-content keeps the
    // color slim). small px so it doesn't leave an obvious gap between the panes
    let handle = rsx! {
        div {
            class: "w-0.5 box-content px-0.5 shrink-0 bg-gray-700 hover:bg-indigo-500 bg-clip-content cursor-col-resize transition-colors",
            onmousedown: move |e| {
                drag.set(Some(DragState {
                    start_x: e.client_coordinates().x,
                    start_width: width(),
                }));
            },
        }
    };

    let panel = rsx! {
        div {
            class: "shrink-0 flex flex-col min-h-0 {props.class}",
            style: "width: {width()}px;",
            {props.children}
        }
    };

    rsx! {
        match side {
            Side::Right => rsx! { {handle} {panel} },
            Side::Left => rsx! { {panel} {handle} },
        }
        if dragging {
            div {
                class: "fixed inset-0 z-50 cursor-col-resize",
                onmousemove: move |e| {
                    if let Some(state) = drag() {
                        // Right grows by dragging the left handle leftward, Left grows by dragging
                        // the right handle rightward
                        let raw = e.client_coordinates().x - state.start_x;
                        let delta = match side {
                            Side::Right => -raw,
                            Side::Left => raw,
                        };
                        width.set((state.start_width + delta).clamp(min, max));
                    }
                },
                onmouseup: move |_| drag.set(None),
                onmouseleave: move |_| drag.set(None),
            }
        }
    }
}
