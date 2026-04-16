use dioxus::prelude::*;

use crate::protocol::{SceneInfo, SceneNode};

#[derive(PartialEq, Clone, Props)]
pub struct SceneTreeProps {
    pub scenes: Vec<SceneInfo>,
    pub selected_path: Option<String>,
    pub on_select: EventHandler<String>,
    pub on_toggle_active: EventHandler<String>,
}

fn node_matches(node: &SceneNode, query: &str) -> bool {
    if node.name.to_lowercase().contains(query) {
        return true;
    }
    node.children.iter().any(|child| node_matches(child, query))
}

fn count_nodes(nodes: &[SceneNode]) -> usize {
    nodes.iter().map(|n| 1 + count_nodes(&n.children)).sum()
}

#[component]
pub fn SceneTree(props: SceneTreeProps) -> Element {
    let mut search = use_signal(String::new);
    let query = search().to_lowercase();
    let searching = !query.is_empty();

    let total: usize = props.scenes.iter().map(|s| count_nodes(&s.objects)).sum();

    rsx! {
        div {
            "data-component": "SceneTree",
            class: "p-4 font-mono text-sm",
            div { class: "flex items-center gap-3 mb-3",
                span { class: "text-gray-500 text-xs",
                    "{props.scenes.len()} scene(s), {total} nodes"
                }
            }
            input {
                class: "w-full px-3 py-1.5 mb-3 bg-gray-700 text-white rounded border border-gray-600 focus:border-indigo-500 focus:outline-none text-sm",
                placeholder: "Search nodes...",
                value: "{search}",
                oninput: move |e| search.set(e.value()),
            }
            for (si, scene) in props.scenes.iter().enumerate() {
                div { key: "{si}", class: "mb-4",
                    h3 { class: "text-base font-bold mb-1",
                        span { class: if scene.is_active { "text-yellow-300" } else { "text-gray-400" },
                            "{scene.name}"
                        }
                        if scene.is_active {
                            span { class: "text-yellow-300 text-xs ml-2", "(active)" }
                        }
                    }
                    div { class: "ml-2",
                        {
                            let filtered: Vec<&SceneNode> = if searching {
                                scene.objects.iter().filter(|n| node_matches(n, &query)).collect()
                            } else {
                                scene.objects.iter().collect()
                            };
                            rsx! {
                                for (i, node) in filtered.iter().enumerate() {
                                    TreeNode {
                                        key: "{i}",
                                        node: (*node).clone(),
                                        filter: if searching { Some(query.clone()) } else { None },
                                        selected_path: props.selected_path.clone(),
                                        on_select: props.on_select,
                                        on_toggle_active: props.on_toggle_active,
                                    }
                                }
                                if searching && filtered.is_empty() {
                                    p { class: "text-gray-500 italic text-xs", "No matches in this scene" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[derive(PartialEq, Clone, Props)]
struct TreeNodeProps {
    node: SceneNode,
    filter: Option<String>,
    selected_path: Option<String>,
    on_select: EventHandler<String>,
    on_toggle_active: EventHandler<String>,
}

#[component]
fn TreeNode(props: TreeNodeProps) -> Element {
    let has_children = !props.node.children.is_empty();
    let mut expanded = use_signal(|| props.filter.is_some());

    let toggle_expand = move |_| {
        if has_children {
            expanded.toggle();
        }
    };

    let toggle_path = props.node.path.clone();
    let on_toggle_active = props.on_toggle_active;
    let toggle_active = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        on_toggle_active.call(toggle_path.clone());
    };

    let icon = if !has_children {
        "  "
    } else if expanded() {
        "▼ "
    } else {
        "▶ "
    };

    let is_selected = props.selected_path.as_ref() == Some(&props.node.path);
    let cursor = if has_children { "cursor-pointer select-none" } else { "select-none" };
    let bg = if is_selected { "bg-indigo-900" } else { "" };

    let visible_children: Vec<&SceneNode> = if let Some(ref query) = props.filter {
        props.node.children.iter().filter(|c| node_matches(c, query)).collect()
    } else {
        props.node.children.iter().collect()
    };

    let name_class = if props.filter.as_ref().map_or(false, |q| props.node.name.to_lowercase().contains(q.as_str())) {
        "text-yellow-300 font-bold"
    } else if props.node.active {
        "text-green-400"
    } else {
        "text-gray-500 line-through"
    };

    let (toggle_icon, toggle_color) = if props.node.active {
        ("●", "text-green-400 hover:text-red-400")
    } else {
        ("○", "text-gray-500 hover:text-green-400")
    };

    let path_for_select = props.node.path.clone();
    let on_select = props.on_select;
    let select_node = move |evt: Event<MouseData>| {
        evt.stop_propagation();
        on_select.call(path_for_select.clone());
    };

    rsx! {
        div {
            div {
                class: "flex items-center py-0.5 hover:bg-gray-700 rounded px-1 {cursor} {bg} group",
                onclick: select_node,
                span {
                    class: "text-gray-500 text-xs w-4",
                    onclick: toggle_expand,
                    "{icon}"
                }
                span { class: "{name_class}", "{props.node.name}" }
                if has_children {
                    span { class: "text-gray-500 ml-2 text-xs",
                        "({props.node.children.len()})"
                    }
                }
                button {
                    class: "ml-auto text-xs {toggle_color} opacity-0 group-hover:opacity-100 px-1",
                    onclick: toggle_active,
                    "{toggle_icon}"
                }
            }
            if has_children && expanded() {
                div { class: "ml-4 border-l border-gray-700 pl-2",
                    for (i, child) in visible_children.iter().enumerate() {
                        TreeNode {
                            key: "{i}",
                            node: (*child).clone(),
                            filter: props.filter.clone(),
                            selected_path: props.selected_path.clone(),
                            on_select: props.on_select,
                            on_toggle_active: props.on_toggle_active,
                        }
                    }
                }
            }
        }
    }
}
