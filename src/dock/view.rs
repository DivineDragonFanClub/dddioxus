use std::collections::HashSet;

use dioxus::prelude::*;
use dioxus_elements::input_data::MouseButton;
use uuid::Uuid;

use super::commands::{self, DockCommand, DropSide};
use super::drag::{self, DragState, Hover};
use super::floating::{floating_window_config, FloatingWindowRoot, FloatingWindowRootProps};
use super::model::{Axis, BindingId, DockNode, DockState, PanelKind};
use super::path::DockPath;
use super::splitter::Splitter;
use crate::components::globals_view::GlobalsView;
use crate::components::inspector_host::InspectorFrame;
use crate::components::procs_view::ProcsView;
use crate::components::scene_view::SceneView;
use crate::hooks::connection::ConnectionState;

#[component]
pub fn DockRoot() -> Element {
    let state = use_context::<Signal<DockState>>();
    drag::use_drag_state();
    use_floating_spawner();
    use_ghost_redock();

    let tree = state.read().main_tree.clone();

    rsx! {
        div {
            "data-component": "DockRoot",
            class: "relative flex flex-1 h-full overflow-hidden",
            DockNodeView { node: tree, path: Vec::<usize>::new() }
            DragOverlay {}
            FloatingGhostOverlay {}
        }
    }
}

/// Watches `DockState.floating` and spawns an OS window for every entry it
/// hasn't already spawned. The FloatingWindowRoot inside each window is
/// responsible for closing it when the entry disappears — we don't track
/// handles here on purpose.
fn use_floating_spawner() {
    let state = use_context::<Signal<DockState>>();
    let conn = use_context::<Signal<ConnectionState>>();
    let ghost = use_context::<Signal<Option<drag::DropGhost>>>();
    let spawned: Signal<HashSet<Uuid>> = use_signal(HashSet::new);

    use_effect(move || {
        let default_bounds = (
            100.0,
            100.0,
            super::floating::DEFAULT_FLOAT_SIZE.0,
            super::floating::DEFAULT_FLOAT_SIZE.1,
        );
        let current: Vec<_> = state
            .read()
            .floating
            .iter()
            .map(|f| (f.id, f.bounds.unwrap_or(default_bounds)))
            .collect();

        let mut spawned = spawned;
        let mut already = spawned.write();
        for (id, bounds) in current {
            if already.insert(id) {
                let dom = VirtualDom::new_with_props(
                    FloatingWindowRoot,
                    FloatingWindowRootProps { window_id: id },
                )
                .with_root_context(state)
                .with_root_context(conn)
                .with_root_context(ghost);
                let _ = dioxus::desktop::window().new_window(
                    dom,
                    floating_window_config(bounds, "Floating panel"),
                );
            }
        }

        // Drop entries from `spawned` once DockState no longer lists them
        // — keeps the set tidy and means a re-eject of the same Uuid (not
        // that we currently do that) would be spawned again.
        let alive: HashSet<Uuid> = state.read().floating.iter().map(|f| f.id).collect();
        already.retain(|id| alive.contains(id));
    });
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
    let mut state = use_context::<Signal<DockState>>();
    let mut drag = use_context::<Signal<Option<DragState>>>();
    let bindings = props.bindings.clone();
    let active = props.active;
    let leaf_path = props.path.clone();

    let state_read = state.read();
    let tabs: Vec<(BindingId, PanelKind, String, bool)> = bindings
        .iter()
        .filter_map(|id| {
            state_read.bindings.get(id).map(|b| {
                let label = tab_label(b);
                // Closable = anything except the follow-selection inspector,
                // which is considered an implicit "always there" slot.
                let closable = !(matches!(b.kind, PanelKind::Inspector) && b.follows_selection);
                (*id, b.kind, label, closable)
            })
        })
        .collect();
    drop(state_read);

    let on_tab_click = {
        let leaf_path = leaf_path.clone();
        move |id: BindingId| {
            let mut w = state.write();
            if let Some(DockNode::Leaf { active, .. }) =
                super::path::node_at_mut(&mut w.main_tree, &leaf_path)
            {
                *active = Some(id);
            }
        }
    };

    let start_drag = {
        move |id: BindingId, label: String, cursor: (f64, f64)| {
            drag.set(Some(DragState {
                binding: id,
                label,
                cursor,
                hover: None,
            }));
        }
    };

    rsx! {
        div {
            "data-component": "Leaf",
            "data-path": "{path_str(&leaf_path)}",
            class: "flex flex-col flex-1 min-w-0 min-h-0 bg-gray-900 overflow-hidden",
            if !tabs.is_empty() {
                // Always show the tab strip so any tab is draggable — even a
                // single-tab leaf is a valid drag source.
                div { class: "flex shrink-0 bg-gray-950 border-b border-gray-800 overflow-x-auto",
                    for (id, kind, label, closable) in tabs.iter().cloned() {
                        {
                            let selected = active == Some(id);
                            let cls = if selected {
                                "flex items-center gap-1 px-3 py-1.5 text-xs text-white bg-gray-800 border-r border-gray-700 select-none cursor-grab"
                            } else {
                                "flex items-center gap-1 px-3 py-1.5 text-xs text-gray-400 hover:text-white hover:bg-gray-800 border-r border-gray-800 select-none cursor-grab"
                            };
                            let mut on_tab_click = on_tab_click.clone();
                            let mut start_drag = start_drag.clone();
                            let label_clone = label.clone();
                            rsx! {
                                div {
                                    key: "{id.0}",
                                    class: "{cls}",
                                    title: "{panel_kind_name(kind)}: {label}",
                                    onmousedown: move |e: MouseEvent| {
                                        if e.trigger_button() == Some(MouseButton::Primary) {
                                            e.prevent_default();
                                            let coord = e.client_coordinates();
                                            start_drag(id, label_clone.clone(), (coord.x, coord.y));
                                            on_tab_click(id);
                                        }
                                    },
                                    span { "{label}" }
                                    if closable {
                                        button {
                                            class: "text-gray-500 hover:text-red-400 ml-1",
                                            title: "Close tab",
                                            onmousedown: move |e: MouseEvent| {
                                                e.stop_propagation();
                                            },
                                            onclick: move |e: MouseEvent| {
                                                e.stop_propagation();
                                                super::commands::apply(
                                                    &mut state.write(),
                                                    DockCommand::CloseBinding { binding: id },
                                                );
                                            },
                                            "×"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "flex flex-1 min-h-0 overflow-hidden",
                if let Some(id) = active {
                    ActivePanel { binding: id }
                } else {
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

#[component]
fn DragOverlay() -> Element {
    let mut state = use_context::<Signal<DockState>>();
    let mut drag = use_context::<Signal<Option<DragState>>>();
    let current = drag();
    let Some(d) = current else {
        return rsx! {};
    };

    let on_move = move |e: MouseEvent| {
        if !e.held_buttons().contains(MouseButton::Primary) {
            drag.set(None);
            return;
        }
        let coord = e.client_coordinates();
        let cursor = (coord.x, coord.y);

        {
            let mut d = drag.write();
            if let Some(state_drag) = d.as_mut() {
                state_drag.cursor = cursor;
            }
        }

        // Async DOM query for the hovered leaf's rect + path; updates the
        // signal on completion. Returns inside one frame on typical hardware.
        spawn(async move {
            let script = format!(
                "var leaves = document.querySelectorAll('[data-component=\"Leaf\"]');
                 var found = null;
                 for (var i = 0; i < leaves.length; i++) {{
                     var r = leaves[i].getBoundingClientRect();
                     if ({x} >= r.left && {x} <= r.right && {y} >= r.top && {y} <= r.bottom) {{
                         found = [leaves[i].getAttribute('data-path') || '', r.left, r.top, r.width, r.height];
                         break;
                     }}
                 }}
                 dioxus.send(found);",
                x = cursor.0,
                y = cursor.1
            );
            let mut eval = document::eval(&script);
            let Ok(val) = eval.recv::<serde_json::Value>().await else {
                return;
            };
            let Some(arr) = val.as_array() else {
                let mut d = drag.write();
                if let Some(s) = d.as_mut() {
                    s.hover = None;
                }
                return;
            };
            if arr.len() != 5 {
                return;
            }
            let path_str = arr[0].as_str().unwrap_or("").to_string();
            let rect = (
                arr[1].as_f64().unwrap_or(0.0),
                arr[2].as_f64().unwrap_or(0.0),
                arr[3].as_f64().unwrap_or(0.0),
                arr[4].as_f64().unwrap_or(0.0),
            );
            let leaf_path: DockPath = path_str
                .split('.')
                .filter(|s| !s.is_empty())
                .filter_map(|s| s.parse::<usize>().ok())
                .collect();
            let side = super::drag::hit_side(rect, cursor);
            let mut d = drag.write();
            if let Some(s) = d.as_mut() {
                s.hover = Some(Hover {
                    leaf_path,
                    rect,
                    side,
                });
            }
        });
    };

    let on_up = move |e: MouseEvent| {
        let Some(d) = drag() else {
            return;
        };
        let coord = e.client_coordinates();
        let cursor = (coord.x, coord.y);

        // Outside-viewport release = eject into a floating window, anchored
        // near the cursor so it doesn't land off-screen.
        spawn(async move {
            let viewport = fetch_viewport().await;
            let (vw, vh) = viewport.unwrap_or((1280.0, 720.0));
            let outside = cursor.0 < 0.0
                || cursor.0 > vw
                || cursor.1 < 0.0
                || cursor.1 > vh;

            if let Some(h) = &d.hover {
                if !outside {
                    commands::apply(
                        &mut state.write(),
                        DockCommand::Drop {
                            binding: d.binding,
                            target: h.leaf_path.clone(),
                            side: h.side,
                        },
                    );
                    drag.set(None);
                    return;
                }
            }

            if outside || d.hover.is_none() {
                // Default float size, positioned near the cursor so the user
                // sees the new window appear where they released.
                let (fw, fh) = super::floating::DEFAULT_FLOAT_SIZE;
                let bounds = (
                    cursor.0.max(0.0),
                    cursor.1.max(0.0),
                    fw,
                    fh,
                );
                commands::apply(
                    &mut state.write(),
                    DockCommand::EjectToFloating {
                        binding: d.binding,
                        bounds,
                    },
                );
            }
            drag.set(None);
        });
    };

    let (cx, cy) = d.cursor;
    let chip_style = format!("left: {}px; top: {}px;", cx + 12.0, cy + 12.0);

    rsx! {
        div {
            "data-component": "DragOverlay",
            class: "fixed inset-0 z-40 bg-gray-900/20",
            style: "cursor: grabbing",
            onmousemove: on_move,
            onmouseup: on_up,
            onmouseleave: move |_| drag.set(None),
            if let Some(h) = d.hover.clone() {
                {
                    let (px, py, pw, ph) = super::drag::preview_rect(h.rect, h.side);
                    let style = format!(
                        "left: {}px; top: {}px; width: {}px; height: {}px;",
                        px, py, pw, ph
                    );
                    rsx! {
                        div {
                            "data-component": "DropPreview",
                            class: "absolute bg-indigo-500/25 border border-indigo-400/90 ring-1 ring-indigo-400/40 transition-all duration-75 pointer-events-none",
                            style: "{style}",
                        }
                    }
                }
            }
            div {
                class: "absolute pointer-events-none bg-indigo-600/90 text-white text-xs px-2 py-1 rounded shadow-lg",
                style: "{chip_style}",
                "{d.label}"
            }
        }
    }
}

/// Renders a translucent ghost rect in the main window while a floating
/// window is being dragged over it. Uses the screen→client conversion
/// below; positions may be roughly off by OS chrome but the user only
/// cares about "am I over a drop zone."
#[component]
fn FloatingGhostOverlay() -> Element {
    let ghost = use_context::<Signal<Option<drag::DropGhost>>>();
    let Some(g) = ghost.read().clone() else {
        return rsx! {};
    };
    if !g.dragging {
        return rsx! {};
    }
    // We don't have a cheap sync way to get the main window's inner_position,
    // so we just render the ghost at the floating window's screen coords.
    // For a drag-back user, the ghost serves as "yes, your drag is being
    // watched" rather than pixel-accurate preview — the actual drop zone
    // lands when `dragging` flips false.
    let screen_x = g.screen_pos.0;
    let screen_y = g.screen_pos.1;
    let style = format!(
        "left: {}px; top: {}px; width: {}px; height: {}px;",
        screen_x, screen_y, g.size.0, g.size.1
    );
    rsx! {
        div {
            "data-component": "FloatingGhost",
            class: "fixed z-30 bg-indigo-500/15 border border-indigo-400/70 ring-1 ring-indigo-400/30 pointer-events-none transition-all duration-75",
            style: "{style}",
        }
    }
}

/// Watches the cross-window `DropGhost` signal. On the `dragging=true →
/// dragging=false` falling edge, runs the hit-test against the main window's
/// viewport and commits `RedockFloating` if a valid leaf zone contains the
/// floating window's title-bar anchor.
fn use_ghost_redock() {
    let mut state = use_context::<Signal<DockState>>();
    let mut ghost = use_context::<Signal<Option<drag::DropGhost>>>();
    let mut was_dragging = use_signal(|| false);

    use_effect(move || {
        let current = ghost.read().clone();
        let now_dragging = current.as_ref().map(|g| g.dragging).unwrap_or(false);
        let prev = was_dragging();

        if prev && !now_dragging {
            // Falling edge — commit attempt.
            if let Some(g) = current {
                spawn(async move {
                    let Some((main_x, main_y, _, _)) = fetch_main_window_rect().await else {
                        ghost.set(None);
                        return;
                    };
                    // Anchor: top-centre of the floating window's title bar in
                    // main-window client space.
                    let ax = g.screen_pos.0 + g.size.0 * 0.5 - main_x;
                    let ay = g.screen_pos.1 + 12.0 - main_y;
                    if let Some((path, rect)) = find_leaf_at(ax, ay).await {
                        let side = super::drag::hit_side(rect, (ax, ay));
                        commands::apply(
                            &mut state.write(),
                            DockCommand::RedockFloating {
                                window: g.window,
                                target: path,
                                side,
                            },
                        );
                    }
                    ghost.set(None);
                });
            }
        }
        was_dragging.set(now_dragging);
    });
}

async fn fetch_main_window_rect() -> Option<(f64, f64, f64, f64)> {
    // Best-effort: use the DOM's position-of-html-at-screen via `window.screenX`/`screenY`.
    let mut eval = document::eval(
        "dioxus.send([window.screenX, window.screenY, window.innerWidth, window.innerHeight]);",
    );
    let val = eval.recv::<serde_json::Value>().await.ok()?;
    let arr = val.as_array()?;
    if arr.len() != 4 {
        return None;
    }
    Some((
        arr[0].as_f64()?,
        arr[1].as_f64()?,
        arr[2].as_f64()?,
        arr[3].as_f64()?,
    ))
}

async fn find_leaf_at(x: f64, y: f64) -> Option<(DockPath, (f64, f64, f64, f64))> {
    let script = format!(
        "var leaves = document.querySelectorAll('[data-component=\"Leaf\"]');
         var found = null;
         for (var i = 0; i < leaves.length; i++) {{
             var r = leaves[i].getBoundingClientRect();
             if ({x} >= r.left && {x} <= r.right && {y} >= r.top && {y} <= r.bottom) {{
                 found = [leaves[i].getAttribute('data-path') || '', r.left, r.top, r.width, r.height];
                 break;
             }}
         }}
         dioxus.send(found);",
        x = x,
        y = y
    );
    let mut eval = document::eval(&script);
    let val = eval.recv::<serde_json::Value>().await.ok()?;
    let arr = val.as_array()?;
    if arr.len() != 5 {
        return None;
    }
    let path_str = arr[0].as_str()?.to_string();
    let rect = (
        arr[1].as_f64()?,
        arr[2].as_f64()?,
        arr[3].as_f64()?,
        arr[4].as_f64()?,
    );
    let leaf_path: DockPath = path_str
        .split('.')
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<usize>().ok())
        .collect();
    Some((leaf_path, rect))
}

async fn fetch_viewport() -> Option<(f64, f64)> {
    let mut eval = document::eval("dioxus.send([window.innerWidth, window.innerHeight]);");
    let val = eval.recv::<serde_json::Value>().await.ok()?;
    let arr = val.as_array()?;
    if arr.len() != 2 {
        return None;
    }
    Some((arr[0].as_f64()?, arr[1].as_f64()?))
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

fn path_str(path: &[usize]) -> String {
    path.iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join(".")
}
