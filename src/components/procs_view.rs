use dioxus::prelude::*;

use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetProcDescsRequest, GetProcDescsResponse, GetProcTreeRequest, GetProcTreeResponse, ProcNode,
};
use crate::rpc;

#[derive(Clone, PartialEq, Debug)]
struct Selection {
    root: String,
    path: Vec<u32>,
    name: String,
    hashcode: i32,
}

const PANEL_MIN_WIDTH: f64 = 200.0;
const PANEL_MAX_WIDTH: f64 = 800.0;
const PANEL_DEFAULT_WIDTH: f64 = 320.0;

#[component]
pub fn ProcsView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetProcTreeResponse, String>>);
    let mut mounted = use_signal(|| false);
    let mut selected = use_signal(|| None::<Selection>);
    let mut panel_width = use_signal(|| PANEL_DEFAULT_WIDTH);
    let mut drag_state = use_signal(|| None::<DragState>);

    let mut fetch = move || {
        if loading() { return; }
        loading.set(true);
        spawn(async move {
            let result = rpc::call(&conn, GetProcTreeRequest).await;
            data.set(Some(result));
            loading.set(false);
        });
    };

    if !mounted() {
        mounted.set(true);
        fetch();
    }

    let on_select = use_callback(move |sel: Selection| selected.set(Some(sel)));
    let dragging = drag_state().is_some();

    rsx! {
        div { class: "flex flex-col h-full relative",
            div { class: "flex items-center gap-2 px-4 py-2 bg-gray-900 border-b border-gray-700",
                h2 { class: "text-white font-bold text-sm", "Proc Tree" }
                button {
                    class: "ml-auto text-white bg-indigo-500 border-0 py-1 px-4 focus:outline-none hover:bg-indigo-600 rounded text-sm",
                    disabled: loading(),
                    onclick: move |_| fetch(),
                    if loading() { "Refreshing..." } else { "Refresh" }
                }
            }
            div { class: "flex flex-1 overflow-hidden",
                div { class: "flex-1 overflow-auto bg-gray-800 p-4 font-mono text-xs",
                    match data().as_ref() {
                        Some(Ok(resp)) => rsx! {
                            for root in resp.roots.iter() {
                                div { key: "{root.label}", class: "mb-4",
                                    h3 { class: "text-yellow-300 font-bold mb-1",
                                        "{root.label} "
                                        span { class: "text-gray-500 text-xs", "({root.children.len()} children)" }
                                    }
                                    if root.children.is_empty() {
                                        p { class: "text-gray-600 italic ml-2", "(empty)" }
                                    } else {
                                        for (i, node) in root.children.iter().enumerate() {
                                            ProcTreeNode {
                                                key: "{i}",
                                                root_label: root.label.clone(),
                                                path: vec![i as u32],
                                                node: node.clone(),
                                                is_next: i > 0,
                                                selected: selected(),
                                                on_select: on_select,
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => rsx! {
                            p { class: "text-red-500", "Error: {err}" }
                        },
                        None => rsx! {
                            p { class: "text-gray-400", "Loading proc tree..." }
                        },
                    }
                }
                if let Some(sel) = selected() {
                    div {
                        class: "w-1 shrink-0 bg-gray-700 hover:bg-indigo-500 cursor-col-resize",
                        onmousedown: move |e| {
                            drag_state.set(Some(DragState {
                                start_x: e.client_coordinates().x,
                                start_width: panel_width(),
                            }));
                        },
                    }
                    DescsPanel { selection: sel, width: panel_width() }
                }
            }
            // Transparent overlay grabs every mouse event while a drag is in progress so
            // the cursor never escapes the splitter, even when crossing iframes/inputs.
            if dragging {
                div {
                    class: "absolute inset-0 z-50 cursor-col-resize",
                    onmousemove: move |e| {
                        if let Some(state) = drag_state() {
                            let delta = state.start_x - e.client_coordinates().x;
                            let new_width = (state.start_width + delta)
                                .clamp(PANEL_MIN_WIDTH, PANEL_MAX_WIDTH);
                            panel_width.set(new_width);
                        }
                    },
                    onmouseup: move |_| drag_state.set(None),
                    onmouseleave: move |_| drag_state.set(None),
                }
            }
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
struct DragState {
    start_x: f64,
    start_width: f64,
}

#[derive(PartialEq, Clone, Props)]
struct ProcTreeNodeProps {
    root_label: String,
    path: Vec<u32>,
    node: ProcNode,
    is_next: bool,
    selected: Option<Selection>,
    on_select: Callback<Selection>,
}

#[component]
fn ProcTreeNode(props: ProcTreeNodeProps) -> Element {
    let has_children = !props.node.children.is_empty();
    let mut expanded = use_signal(|| false);

    let caret = if !has_children {
        "  "
    } else if expanded() {
        "▼ "
    } else {
        "▶ "
    };

    let mut toggle_expand = move |_: Event<MouseData>| {
        if has_children {
            expanded.toggle();
        }
    };

    let is_selected = props
        .selected
        .as_ref()
        .map(|s| s.root == props.root_label && s.path == props.path)
        .unwrap_or(false);

    let row_bg = if is_selected { "bg-indigo-900" } else { "" };

    let select = {
        let root_label = props.root_label.clone();
        let path = props.path.clone();
        let name = props.node.name.clone();
        let hashcode = props.node.hashcode;
        let on_select = props.on_select;
        move |_: Event<MouseData>| {
            on_select.call(Selection {
                root: root_label.clone(),
                path: path.clone(),
                name: name.clone(),
                hashcode,
            });
        }
    };

    rsx! {
        div {
            div {
                class: "flex items-baseline gap-2 py-0.5 px-1 hover:bg-gray-700 rounded cursor-pointer select-none {row_bg}",
                onclick: select,
                span {
                    class: "text-gray-500 w-4",
                    onclick: move |e: Event<MouseData>| {
                        e.stop_propagation();
                        toggle_expand(e);
                    },
                    "{caret}"
                }
                if props.is_next {
                    span { class: "text-indigo-400 text-[10px] w-3", "↓" }
                } else {
                    span { class: "w-3" }
                }
                span { class: "text-gray-200", "{props.node.name}" }
                span { class: "text-gray-500 text-[10px]", "#{props.node.hashcode}" }
                span { class: "text-cyan-400 text-[10px]", "desc {props.node.desc_index}" }
                if has_children {
                    span { class: "text-gray-500 text-[10px]", "({props.node.children.len()})" }
                }
            }
            if has_children && expanded() {
                div { class: "ml-4 border-l border-gray-700 pl-2",
                    for (i, child) in props.node.children.iter().enumerate() {
                        ProcTreeNode {
                            key: "{i}",
                            root_label: props.root_label.clone(),
                            path: {
                                let mut p = props.path.clone();
                                p.push(i as u32);
                                p
                            },
                            node: child.clone(),
                            is_next: i > 0,
                            selected: props.selected.clone(),
                            on_select: props.on_select,
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct DescsPanelProps {
    selection: Selection,
    width: f64,
}

#[component]
fn DescsPanel(props: DescsPanelProps) -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut data = use_signal(|| None::<Result<GetProcDescsResponse, String>>);
    let mut last_key = use_signal(String::new);

    let key = format!("{}/{:?}", props.selection.root, props.selection.path);
    if last_key() != key {
        last_key.set(key);
        let req = GetProcDescsRequest {
            root: props.selection.root.clone(),
            path: props.selection.path.clone(),
        };
        spawn(async move {
            let result = rpc::call(&conn, req).await;
            data.set(Some(result));
        });
    }

    let width = props.width as i32;

    rsx! {
        div {
            class: "shrink-0 bg-gray-900 overflow-auto font-mono text-xs",
            style: "width: {width}px",
            div { class: "px-3 py-2 bg-gray-800 border-b border-gray-700",
                h3 { class: "text-white font-bold text-sm", "{props.selection.name}" }
                p { class: "text-gray-500 text-xs",
                    "#{props.selection.hashcode} • {props.selection.root} / {props.selection.path:?}"
                }
            }
            div { class: "p-3",
                match data().as_ref() {
                    Some(Ok(resp)) => {
                        let descs = resp.descs.clone();
                        let current = resp.desc_index;
                        rsx! {
                            h4 { class: "text-indigo-300 font-bold mb-2",
                                "Descs ({descs.len()})"
                            }
                            if descs.is_empty() {
                                p { class: "text-gray-600 italic", "(no descriptors)" }
                            }
                            for (i, info) in descs.iter().enumerate() {
                                div {
                                    key: "{i}",
                                    class: if i as i32 == current {
                                        "flex flex-col py-0.5 px-1 rounded bg-indigo-900 text-yellow-300"
                                    } else {
                                        "flex flex-col py-0.5 px-1 rounded hover:bg-gray-800 text-gray-200"
                                    },
                                    div { class: "flex items-baseline gap-2",
                                        span { class: "text-gray-500 w-8 text-right", "{i}" }
                                        span { "{info.kind}" }
                                        if i as i32 == current {
                                            span { class: "ml-auto text-[10px] text-yellow-400", "← current" }
                                        }
                                    }
                                    if let Some(m) = info.method.clone() {
                                        span { class: "ml-10 text-cyan-400 text-[10px] truncate", "{m}" }
                                    }
                                    if let Some(l) = info.label.clone() {
                                        span { class: "ml-10 text-pink-400 text-[10px] truncate", "label = {l}" }
                                    }
                                }
                            }
                        }
                    }
                    Some(Err(err)) => rsx! {
                        p { class: "text-red-500", "Error: {err}" }
                    },
                    None => rsx! {
                        p { class: "text-gray-400", "Loading..." }
                    },
                }
            }
        }
    }
}
