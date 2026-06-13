use std::time::Duration;

use dioxus::prelude::*;

use super::scene_view::RevealRequest;
use crate::components::ui::{EmptyState, SearchField, SectionLabel, StateKind};
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
    // Debounced copy of `search`: the input updates `search` instantly (so typing
    // stays responsive), but the recursive tree filter only runs off `applied`,
    // ~150ms after the user stops typing. `debounce_epoch` cancels superseded keystrokes.
    let mut applied = use_signal(String::new);
    let mut debounce_epoch = use_signal(|| 0u64);
    let query = applied().to_lowercase();
    let searching = !query.is_empty();

    let total: usize = props.scenes.iter().map(|s| count_nodes(&s.objects)).sum();

    rsx! {
        div {
            "data-component": "SceneTree",
            class: "p-3 font-mono text-sm",
            // sticky toolbar with scene count + search
            div {
                class: "sticky top-0 z-10 bg-gray-800/95 -mx-3 -mt-3 mb-1 px-3 pt-3 pb-2 \
                        border-b border-gray-700/70",
                SectionLabel { label: "{props.scenes.len()} scene(s), {total} nodes", class: "mb-2" }
                SearchField {
                    value: search(),
                    placeholder: "Search nodes\u{2026}",
                    on_input: move |val: String| {
                        search.set(val.clone());
                        let epoch = debounce_epoch() + 1;
                        debounce_epoch.set(epoch);
                        spawn(async move {
                            tokio::time::sleep(Duration::from_millis(150)).await;
                            if debounce_epoch() == epoch {
                                applied.set(val);
                            }
                        });
                    },
                }
            }
            for (si, scene) in props.scenes.iter().enumerate() {
                div { key: "{si}", class: "mb-4",
                    h3 { class: "text-sm font-semibold mb-1 flex items-center gap-2",
                        span { class: if scene.is_active { "text-amber-300" } else { "text-gray-400" },
                            "{scene.name}"
                        }
                        if scene.is_active {
                            span { class: "text-amber-300 text-xs font-normal", "(active)" }
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
                                    EmptyState {
                                        kind: StateKind::Empty,
                                        message: "No matches in this scene",
                                        compact: true,
                                    }
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

    // Effective open state is reactive: open while searching, or when the selected
    // node is this node or somewhere in its subtree (so clearing the search keeps the
    // selected node's path revealed -- its siblings via the open ancestors, its children
    // via itself). A manual click overrides until the user toggles again.
    let reveals_selection = props.selected_path.as_deref().map_or(false, |sel| {
        props.node.path == sel || sel.starts_with(&format!("{}/", props.node.path))
    });
    let mut user_open = use_signal(|| None::<bool>);

    // One-shot reveal: when the Inspector's "see in tree" link bumps the reveal
    // counter, nodes on the selected path drop their manual collapse so the path
    // force-expands for that click only (the user can still re-collapse afterward).
    let mut seen_reveal = use_signal(|| 0u32);
    if let Some(RevealRequest(nonce)) = try_consume_context::<RevealRequest>() {
        if nonce() != seen_reveal() {
            seen_reveal.set(nonce());
            if reveals_selection {
                user_open.set(None);
            }
        }
    }

    // Manual toggle wins; otherwise open while searching or on the selected path.
    let auto_open = props.filter.is_some() || reveals_selection;
    let is_open = user_open().unwrap_or(auto_open);

    let toggle_expand = move |evt: Event<MouseData>| {
        if has_children {
            evt.stop_propagation();
            user_open.set(Some(!is_open));
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
    } else if is_open {
        "▼ "
    } else {
        "▶ "
    };

    let is_selected = props.selected_path.as_ref() == Some(&props.node.path);
    let cursor = if has_children { "cursor-pointer select-none" } else { "select-none" };

    // selected highlight matches the kit's ListRow active state
    let row_bg = if is_selected {
        "bg-indigo-500/15 ring-1 ring-inset ring-indigo-500/40"
    } else {
        "hover:bg-gray-700/50"
    };

    let visible_children: Vec<&SceneNode> = if let Some(ref query) = props.filter {
        props.node.children.iter().filter(|c| node_matches(c, query)).collect()
    } else {
        props.node.children.iter().collect()
    };

    // highlight search matches in amber; active = green, inactive = struck-through gray
    let name_class = if props.filter.as_ref().map_or(false, |q| props.node.name.to_lowercase().contains(q.as_str())) {
        "text-amber-300 font-semibold"
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
                class: "flex items-center py-0.5 rounded px-1 transition-colors {row_bg} {cursor} group",
                "data-tree-selected": "{is_selected}",
                onclick: select_node,
                span {
                    class: "text-gray-500 text-xs w-4 shrink-0",
                    onclick: toggle_expand,
                    "{icon}"
                }
                span { class: "{name_class} truncate flex-1", "{props.node.name}" }
                if has_children {
                    span { class: "text-gray-600 ml-1.5 text-xs shrink-0",
                        "({props.node.children.len()})"
                    }
                }
                button {
                    class: "ml-auto text-xs {toggle_color} opacity-0 group-hover:opacity-100 px-1 shrink-0",
                    onclick: toggle_active,
                    "{toggle_icon}"
                }
            }
            if has_children && is_open {
                div { class: "ml-4 border-l border-gray-700/60 pl-2",
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
