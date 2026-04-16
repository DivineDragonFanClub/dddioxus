use std::time::{Duration, Instant};

use dioxus::desktop::tao::event::{Event, WindowEvent};
use dioxus::desktop::{
    use_wry_event_handler, window, Config as WindowConfig, WindowBuilder,
};
use dioxus::prelude::*;
use uuid::Uuid;

use super::commands::{self, DockCommand};
use super::drag::{self, DropGhost};
use super::model::{DockNode, DockState};
use super::view::DockNodeView;

/// Root of a secondary window: renders a custom title bar + the floating
/// window's dock subtree. Takes its identity as a prop (the Uuid inside
/// `DockState.floating`) rather than inheriting it from context.
#[derive(PartialEq, Clone, Props)]
pub struct FloatingWindowRootProps {
    pub window_id: Uuid,
}

#[component]
pub fn FloatingWindowRoot(props: FloatingWindowRootProps) -> Element {
    let mut state = use_context::<Signal<DockState>>();
    let mut ghost = use_context::<Signal<Option<DropGhost>>>();
    let window_id = props.window_id;
    let state_read = state.read();
    let Some(fw) = state_read.floating.iter().find(|f| f.id == window_id) else {
        // The floating entry was removed (user closed or redocked).
        // Close this window so it doesn't linger with empty state.
        let _ = &state_read;
        drop(state_read);
        window().close();
        return rsx! {};
    };
    let tree = fw.tree.clone();
    let title = floating_window_title(&fw.tree, &state_read);
    drop(state_read);

    // Publish `Moved` events to both the bounds store (for persistence) and
    // the cross-window ghost signal (for drag-back preview).
    let owned_id = window().id();
    use_wry_event_handler(move |event, _| {
        if let Event::WindowEvent {
            event: WindowEvent::Moved(pos),
            window_id: wid,
            ..
        } = event
        {
            if *wid == owned_id {
                let size = {
                    let guard = state.read();
                    let current = guard.floating.iter().find(|f| f.id == window_id);
                    let (_, _, w, h) = current
                        .and_then(|f| f.bounds)
                        .unwrap_or((0.0, 0.0, 400.0, 300.0));
                    (w, h)
                };
                let screen_pos = (pos.x as f64, pos.y as f64);
                commands::apply(
                    &mut state.write(),
                    DockCommand::UpdateFloatingBounds {
                        window: window_id,
                        bounds: (screen_pos.0, screen_pos.1, size.0, size.1),
                    },
                );
                ghost.set(Some(DropGhost {
                    window: window_id,
                    screen_pos,
                    size,
                    last_move: Instant::now(),
                    dragging: true,
                }));
            }
        }
    });

    // Debounce the active flag 150 ms after the last `Moved`. The falling
    // edge of `dragging` is the main window's cue to commit the re-dock.
    let mut ghost_signal = ghost;
    use_effect(move || {
        let snapshot = ghost_signal.read().clone();
        if let Some(g) = snapshot {
            if g.window == window_id && g.dragging {
                let ts = g.last_move;
                spawn(async move {
                    tokio::time::sleep(Duration::from_millis(150)).await;
                    let mut ghost = ghost_signal;
                    let still_same = ghost
                        .peek()
                        .as_ref()
                        .map(|cur| cur.last_move == ts && cur.dragging)
                        .unwrap_or(false);
                    if still_same {
                        // Flip `dragging` to false; main window's effect
                        // handles the rest.
                        let mut cur = ghost.write();
                        if let Some(g) = cur.as_mut() {
                            g.dragging = false;
                        }
                    }
                });
            }
        }
    });

    rsx! {
        document::Stylesheet { href: crate::TAILWIND }
        document::Style { "html, body {{ margin: 0; height: 100%; overflow: hidden; background: #111827; }}" }
        div { class: "flex flex-col h-screen bg-gray-900 text-white",
            // Custom title bar — click-and-drag calls `window().drag()` to
            // kick an OS-level window move.
            div {
                "data-component": "FloatingTitleBar",
                class: "flex items-center justify-between shrink-0 bg-gray-950 border-b border-gray-700 px-3 py-1.5 select-none",
                style: "cursor: grab",
                onmousedown: move |_| {
                    let _ = window().drag();
                },
                span { class: "text-xs text-gray-300 truncate", "{title}" }
                button {
                    class: "ml-2 text-gray-500 hover:text-red-400 text-sm",
                    title: "Close floating window",
                    onmousedown: move |e: MouseEvent| {
                        // Don't start an OS drag when the user clicks ×.
                        e.stop_propagation();
                    },
                    onclick: move |e: MouseEvent| {
                        e.stop_propagation();
                        commands::apply(
                            &mut state.write(),
                            DockCommand::CloseFloating { window: window_id },
                        );
                        window().close();
                    },
                    "×"
                }
            }
            div { class: "relative flex flex-1 overflow-hidden",
                DockNodeView { node: tree, path: Vec::<usize>::new() }
            }
        }
    }
}

/// Default window size for a freshly-ejected panel.
pub const DEFAULT_FLOAT_SIZE: (f64, f64) = (520.0, 420.0);

/// Build a `Config` for a borderless floating window anchored at `bounds`
/// (x, y, w, h in logical pixels).
pub fn floating_window_config(bounds: (f64, f64, f64, f64), title: &str) -> WindowConfig {
    let (x, y, w, h) = bounds;
    let win = WindowBuilder::new()
        .with_decorations(false)
        .with_title(title)
        .with_inner_size(dioxus::desktop::tao::dpi::LogicalSize::new(w, h))
        .with_position(dioxus::desktop::tao::dpi::LogicalPosition::new(x, y));
    WindowConfig::new().with_window(win)
}

fn floating_window_title(tree: &DockNode, state: &DockState) -> String {
    let first = first_binding(tree);
    match first {
        Some(id) => match state.bindings.get(&id) {
            Some(b) => match b.kind {
                super::model::PanelKind::Scene => "Scene".into(),
                super::model::PanelKind::Globals => "Globals".into(),
                super::model::PanelKind::Procs => "Procs".into(),
                super::model::PanelKind::Inspector => b
                    .path
                    .clone()
                    .unwrap_or_else(|| "Inspector".into()),
            },
            None => "Floating".into(),
        },
        None => "Floating".into(),
    }
}

fn first_binding(node: &DockNode) -> Option<super::model::BindingId> {
    match node {
        DockNode::Leaf { bindings, .. } => bindings.first().copied(),
        DockNode::Split { first, .. } => first_binding(first),
    }
}

