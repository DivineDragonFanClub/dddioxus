use dioxus::prelude::*;

use crate::components::resizable_panel::{ResizablePanel, Side};
use crate::components::ui::{Button, Card, EmptyState, Page, PanelHeader, StateKind};
use crate::hooks::connection::ConnectionState;
use crate::protocol::{
    GetProcDescsRequest, GetProcDescsResponse, GetProcTreeRequest, GetProcTreeResponse, ProcNode,
};
use crate::rpc;

#[derive(Clone, PartialEq, Debug)]
pub struct Selection {
    pub root: String,
    pub path: Vec<u32>,
    pub name: String,
    pub hashcode: i32,
}

#[component]
pub fn ProcsView() -> Element {
    let conn = use_context::<Signal<ConnectionState>>();
    let mut loading = use_signal(|| false);
    let mut data = use_signal(|| None::<Result<GetProcTreeResponse, String>>);
    let mut mounted = use_signal(|| false);
    let mut selected = use_signal(|| None::<Selection>);
    let mut descs_data = use_signal(|| None::<Result<GetProcDescsResponse, String>>);
    let mut last_descs_key = use_signal(String::new);

    let mut fetch = move || {
        if loading() {
            return;
        }
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

    let current_selected = selected();
    let current_key = match current_selected.as_ref() {
        Some(sel) => format!("{}/{:?}", sel.root, sel.path),
        None => String::new(),
    };
    if last_descs_key() != current_key {
        last_descs_key.set(current_key);
        match current_selected.as_ref() {
            Some(sel) => {
                let req = GetProcDescsRequest {
                    root: sel.root.clone(),
                    path: sel.path.clone(),
                };
                descs_data.set(None);
                spawn(async move {
                    let result = rpc::call(&conn, req).await;
                    descs_data.set(Some(result));
                });
            }
            None => descs_data.set(None),
        }
    }

    let on_select = use_callback(move |sel: Selection| selected.set(Some(sel)));

    rsx! {
        ProcsPanel {
            data: data(),
            loading: loading(),
            selected: selected(),
            descs_data: descs_data(),
            on_refresh: move |_| fetch(),
            on_select: on_select,
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ProcsPanelProps {
    pub data: Option<Result<GetProcTreeResponse, String>>,
    pub loading: bool,
    pub selected: Option<Selection>,
    pub descs_data: Option<Result<GetProcDescsResponse, String>>,
    pub on_refresh: EventHandler<()>,
    pub on_select: Callback<Selection>,
}

#[component]
pub fn ProcsPanel(props: ProcsPanelProps) -> Element {
    let on_refresh = props.on_refresh;
    let on_select = props.on_select;

    rsx! {
        Page { row: true,
            ResizablePanel {
                side: Side::Left,
                class: "bg-gray-800/80 border border-gray-700/70 rounded-xl shadow-lg shadow-black/30 overflow-hidden",
                default_width: 320.0,
                min_width: 200.0,
                max_width: 800.0,
                // card header: title + refresh button
                div { class: "flex items-center gap-2 px-3 py-2 border-b border-gray-700/70 bg-gray-900/40 shrink-0",
                    h3 { class: "text-white font-semibold text-sm truncate", "Proc Tree" }
                    div { class: "ml-auto flex items-center gap-2",
                        Button {
                            disabled: props.loading,
                            onclick: move |_| on_refresh.call(()),
                            if props.loading { "Refreshing\u{2026}" } else { "Refresh" }
                        }
                    }
                }
                // tree body
                div { class: "flex-1 overflow-auto p-4 font-mono text-xs",
                    match props.data.as_ref() {
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
                                                selected: props.selected.clone(),
                                                on_select: on_select,
                                            }
                                        }
                                    }
                                }
                            }
                        },
                        Some(Err(err)) => rsx! {
                            EmptyState { kind: StateKind::Error, message: "Error: {err}" }
                        },
                        None => rsx! {
                            EmptyState { kind: StateKind::Loading, message: "Loading proc tree\u{2026}" }
                        },
                    }
                }
            }
            // right pane: desc inspector, only shown when something is selected
            if let Some(sel) = props.selected.clone() {
                DescsPanel {
                    selection: sel,
                    width: 0.0,
                    data: props.descs_data.clone(),
                }
            } else {
                Card {
                    class: "flex-1",
                    padded: false,
                    PanelHeader { title: "Descriptors", subtitle: "No selection".to_string() }
                    EmptyState {
                        kind: StateKind::Empty,
                        message: "Select a proc node in the tree to see its descriptors.",
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct ProcTreeNodeProps {
    pub root_label: String,
    pub path: Vec<u32>,
    pub node: ProcNode,
    pub is_next: bool,
    pub selected: Option<Selection>,
    pub on_select: Callback<Selection>,
}

#[component]
pub fn ProcTreeNode(props: ProcTreeNodeProps) -> Element {
    let has_children = !props.node.children.is_empty();
    let mut expanded = use_signal(|| false);

    let caret = if !has_children {
        "  "
    } else if expanded() {
        "\u{25BC} "
    } else {
        "\u{25B6} "
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

    let row_bg = if is_selected {
        "bg-indigo-500/15 ring-1 ring-indigo-500/40"
    } else {
        "hover:bg-gray-700/50"
    };

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
                class: "flex items-baseline gap-2 py-0.5 px-1 rounded cursor-pointer select-none transition-colors {row_bg}",
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
                    span { class: "text-indigo-400 text-[10px] w-3", "\u{2193}" }
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
pub struct DescsPanelProps {
    pub selection: Selection,
    // width is no longer applied as a style (ResizablePanel on the left controls the split now),
    // kept in the signature so callers that were already passing it don't need to change
    pub width: f64,
    pub data: Option<Result<GetProcDescsResponse, String>>,
}

#[component]
pub fn DescsPanel(props: DescsPanelProps) -> Element {
    rsx! {
        div {
            class: "flex-1 flex flex-col min-h-0 bg-gray-800/80 border border-gray-700/70 rounded-xl shadow-lg shadow-black/30 overflow-hidden font-mono text-xs",
            PanelHeader {
                title: props.selection.name.clone(),
                subtitle: format!("#{} \u{00B7} {} / {:?}", props.selection.hashcode, props.selection.root, props.selection.path),
            }
            div { class: "flex-1 overflow-auto p-3",
                match props.data.as_ref() {
                    Some(Ok(resp)) => {
                        let descs = resp.descs.clone();
                        let current = resp.desc_index;
                        rsx! {
                            h4 { class: "text-indigo-300 font-bold mb-2",
                                "Descs ({descs.len()})"
                            }
                            if descs.is_empty() {
                                EmptyState { kind: StateKind::Empty, message: "(no descriptors)", compact: true }
                            }
                            for (i, info) in descs.iter().enumerate() {
                                div {
                                    key: "{i}",
                                    class: if i as i32 == current {
                                        "flex flex-col py-0.5 px-1 rounded bg-indigo-500/15 ring-1 ring-indigo-500/40 text-yellow-300"
                                    } else {
                                        "flex flex-col py-0.5 px-1 rounded hover:bg-gray-700/50 text-gray-200"
                                    },
                                    div { class: "flex items-baseline gap-2",
                                        span { class: "text-gray-500 w-8 text-right", "{i}" }
                                        span { "{info.kind}" }
                                        if i as i32 == current {
                                            span { class: "ml-auto text-[10px] text-yellow-400", "\u{2190} current" }
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
                        EmptyState { kind: StateKind::Error, message: "Error: {err}" }
                    },
                    None => rsx! {
                        EmptyState { kind: StateKind::Loading, message: "Loading\u{2026}" }
                    },
                }
            }
        }
    }
}
