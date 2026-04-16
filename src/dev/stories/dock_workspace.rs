use dioxus::prelude::*;

use crate::dock::drag::DropGhost;
use crate::dock::view::DockRoot;
use crate::dock::{self, DockCommand, DockState, PanelKind};
use crate::hooks::connection::ConnectionState;

/// Mounts `DockRoot` with fake contexts so the user can exercise dragging,
/// splitting, resizing, ejecting, and redocking without a live game.
///
/// The inner panel bodies (Scene tree, Transform inspector, etc.) will
/// show their disconnected error states — this page is a docking sandbox,
/// not a panel sandbox.
#[component]
pub fn DevDockWorkspace() -> Element {
    // Story-local signals provided at this scope shadow any same-typed
    // signal provided higher up (e.g. the App's autosaver-backed
    // DockState), so mutations here never touch `~/.dddioxus_layout.json`.
    let state = use_signal(DockState::default_layout);
    let conn = use_signal(|| ConnectionState::Disconnected);
    let ghost = use_signal(|| None::<DropGhost>);
    use_hook(|| {
        provide_context(state);
        provide_context(conn);
        provide_context(ghost);
    });

    let mut state = state;
    let mut open = move |kind: PanelKind| {
        dock::apply(&mut state.write(), DockCommand::OpenPanel { kind });
    };
    let open_new_inspector = move |_| {
        // Force-create a second inspector by temporarily flipping the
        // existing follow flag off, calling OpenPanel (which only creates
        // when no binding of that kind exists), and flipping it back.
        let mut w = state.write();
        // Instead, build and append the binding directly — this matches
        // the "pin" flow but with no source path to clone.
        use crate::dock::{Binding, BindingId};
        let b = Binding {
            id: BindingId::new(),
            kind: PanelKind::Inspector,
            path: Some(format!("/storybook/inspector-{}", w.bindings.len())),
            follows_selection: false,
            title: None,
        };
        let id = b.id;
        w.bindings.insert(id, b);
        // Append to the first leaf we can find.
        fn append(
            node: &mut crate::dock::DockNode,
            id: crate::dock::BindingId,
        ) {
            match node {
                crate::dock::DockNode::Leaf { bindings, active } => {
                    bindings.push(id);
                    if active.is_none() {
                        *active = Some(id);
                    }
                }
                crate::dock::DockNode::Split { first, .. } => append(first, id),
            }
        }
        append(&mut w.main_tree, id);
    };
    let reset = move |_| {
        dock::apply(&mut state.write(), DockCommand::ResetLayout);
    };

    rsx! {
        div { class: "flex flex-col h-full overflow-hidden",
            div { class: "shrink-0 bg-gray-950 border-b border-gray-800 p-3 space-y-2",
                h1 { class: "text-white font-bold text-sm", "Dock workspace sandbox" }
                p { class: "text-gray-500 text-xs max-w-2xl",
                    "Inner panels render their disconnected error states — this page tests docking mechanics (drag, split, resize, eject, redock) only. Open extra inspectors to have more tabs to move around."
                }
                div { class: "flex gap-2 text-xs",
                    button {
                        class: "px-3 py-1 bg-indigo-600 hover:bg-indigo-500 rounded text-white",
                        onclick: reset,
                        "Reset layout"
                    }
                    button {
                        class: "px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-gray-200",
                        onclick: move |_| open(PanelKind::Scene),
                        "Open Scene"
                    }
                    button {
                        class: "px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-gray-200",
                        onclick: move |_| open(PanelKind::Globals),
                        "Open Globals"
                    }
                    button {
                        class: "px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-gray-200",
                        onclick: move |_| open(PanelKind::Procs),
                        "Open Procs"
                    }
                    button {
                        class: "px-3 py-1 bg-gray-800 hover:bg-gray-700 rounded text-gray-200",
                        onclick: open_new_inspector,
                        "Add Inspector tab"
                    }
                }
            }
            div { class: "flex-1 overflow-hidden",
                DockRoot {}
            }
        }
    }
}
