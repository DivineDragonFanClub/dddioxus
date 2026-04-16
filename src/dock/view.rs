use dioxus::prelude::*;

use super::model::{Axis, BindingId, DockNode, DockState, PanelKind};
use super::path::DockPath;
use super::splitter::Splitter;
use crate::components::globals_view::GlobalsView;
use crate::components::inspector_host::InspectorFrame;
use crate::components::procs_view::ProcsView;
use crate::components::scene_view::SceneView;

#[component]
pub fn DockRoot() -> Element {
    let state = use_context::<Signal<DockState>>();
    let tree = state.read().main_tree.clone();
    rsx! {
        div {
            "data-component": "DockRoot",
            class: "flex flex-1 h-full overflow-hidden",
            DockNodeView { node: tree, path: Vec::<usize>::new() }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct DockNodeViewProps {
    pub node: DockNode,
    pub path: DockPath,
}

#[component]
pub fn DockNodeView(props: DockNodeViewProps) -> Element {
    match props.node {
        DockNode::Leaf { bindings, active } => rsx! {
            LeafView { path: props.path, bindings: bindings, active: active }
        },
        DockNode::Split {
            dir,
            ratio,
            first,
            second,
        } => {
            let mut first_path = props.path.clone();
            first_path.push(0);
            let mut second_path = props.path.clone();
            second_path.push(1);
            rsx! {
                SplitView {
                    path: props.path,
                    dir: dir,
                    ratio: ratio,
                    first: *first,
                    first_path: first_path,
                    second: *second,
                    second_path: second_path,
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct SplitViewProps {
    pub path: DockPath,
    pub dir: Axis,
    pub ratio: f32,
    pub first: DockNode,
    pub first_path: DockPath,
    pub second: DockNode,
    pub second_path: DockPath,
}

#[component]
pub fn SplitView(props: SplitViewProps) -> Element {
    let horizontal = matches!(props.dir, Axis::Horizontal);
    let outer_class = if horizontal {
        "flex flex-row flex-1 h-full overflow-hidden"
    } else {
        "flex flex-col flex-1 h-full overflow-hidden"
    };
    let pct = (props.ratio * 100.0).clamp(5.0, 95.0);
    let rest = 100.0 - pct;
    let first_style = if horizontal {
        format!("width: {:.3}%;", pct)
    } else {
        format!("height: {:.3}%;", pct)
    };
    let second_style = if horizontal {
        format!("width: {:.3}%;", rest)
    } else {
        format!("height: {:.3}%;", rest)
    };

    rsx! {
        div { "data-component": "Split", class: "{outer_class}",
            div {
                class: "relative flex overflow-hidden",
                style: "{first_style}",
                DockNodeView { node: props.first, path: props.first_path }
            }
            Splitter { path: props.path.clone(), axis: props.dir }
            div {
                class: "relative flex overflow-hidden",
                style: "{second_style}",
                DockNodeView { node: props.second, path: props.second_path }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
pub struct LeafViewProps {
    pub path: DockPath,
    pub bindings: Vec<BindingId>,
    pub active: Option<BindingId>,
}

#[component]
pub fn LeafView(props: LeafViewProps) -> Element {
    let state = use_context::<Signal<DockState>>();
    let bindings = props.bindings.clone();
    let active = props.active;

    let state_read = state.read();
    let tabs: Vec<(BindingId, PanelKind, String)> = bindings
        .iter()
        .filter_map(|id| {
            state_read.bindings.get(id).map(|b| {
                let label = tab_label(b);
                (*id, b.kind, label)
            })
        })
        .collect();
    let active_kind = active.and_then(|id| state_read.bindings.get(&id).map(|b| b.kind));
    drop(state_read);

    let on_tab_click = {
        let mut state = state;
        let path = props.path.clone();
        move |id: BindingId| {
            let mut w = state.write();
            if let Some(node) = super::path::node_at_mut(&mut w.main_tree, &path) {
                if let DockNode::Leaf { active, .. } = node {
                    *active = Some(id);
                }
            }
        }
    };

    rsx! {
        div {
            "data-component": "Leaf",
            class: "flex flex-col flex-1 min-w-0 min-h-0 bg-gray-900 overflow-hidden",
            if tabs.len() > 1 {
                div { class: "flex shrink-0 bg-gray-950 border-b border-gray-800 overflow-x-auto",
                    for (id, kind, label) in tabs.iter().cloned() {
                        {
                            let selected = active == Some(id);
                            let cls = if selected {
                                "px-3 py-1.5 text-xs text-white bg-gray-800 border-r border-gray-700"
                            } else {
                                "px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-800 border-r border-gray-800"
                            };
                            let mut on_tab_click = on_tab_click.clone();
                            rsx! {
                                button {
                                    key: "{id.0}",
                                    class: "{cls}",
                                    title: "{panel_kind_name(kind)}: {label}",
                                    onclick: move |_| on_tab_click(id),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex flex-1 min-h-0 overflow-hidden",
                if let Some(id) = active {
                    ActivePanel { binding: id }
                } else if !bindings.is_empty() {
                    p { class: "p-4 text-gray-500 text-sm italic", "Empty leaf" }
                } else if let Some(_kind) = active_kind {
                    p { class: "p-4 text-gray-500 text-sm italic", "Empty leaf" }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct ActivePanelProps {
    binding: BindingId,
}

#[component]
fn ActivePanel(props: ActivePanelProps) -> Element {
    let state = use_context::<Signal<DockState>>();
    let kind = state.read().bindings.get(&props.binding).map(|b| b.kind);
    match kind {
        Some(PanelKind::Scene) => rsx! { SceneView {} },
        Some(PanelKind::Globals) => rsx! { GlobalsView {} },
        Some(PanelKind::Procs) => rsx! { ProcsView {} },
        Some(PanelKind::Inspector) => {
            let binding = state.read().bindings.get(&props.binding).cloned();
            match binding {
                Some(b) => rsx! { InspectorFrame { binding: b } },
                None => rsx! {},
            }
        }
        None => rsx! {},
    }
}

fn tab_label(b: &super::model::Binding) -> String {
    if let Some(title) = &b.title {
        return title.clone();
    }
    match b.kind {
        PanelKind::Scene => "Scene".into(),
        PanelKind::Globals => "Globals".into(),
        PanelKind::Procs => "Procs".into(),
        PanelKind::Inspector => {
            if let Some(path) = &b.path {
                path.rsplit('/').next().unwrap_or(path).to_string()
            } else {
                "Inspector".into()
            }
        }
    }
}

fn panel_kind_name(kind: PanelKind) -> &'static str {
    match kind {
        PanelKind::Scene => "Scene",
        PanelKind::Globals => "Globals",
        PanelKind::Procs => "Procs",
        PanelKind::Inspector => "Inspector",
    }
}
