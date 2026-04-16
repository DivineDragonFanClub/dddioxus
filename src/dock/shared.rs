//! Cross-window reactive state.
//!
//! **Key insight**: Dioxus 0.7 signals are owned by a runtime that is shared
//! across all `VirtualDom`s spawned from the same process (every `new_window`
//! call). When a `Signal<T>` is provided via `with_root_context` to a secondary
//! window's VirtualDom, **writes propagate to every subscribed scope in every
//! VirtualDom**. No Arc<RwLock>, no broadcast channel, no manual version
//! bumping required.
//!
//! The docking planning doc (`planning/phase-5-cross-window-state.md`)
//! proposed a `SharedDockState` wrapper around `Arc<RwLock<DockState>>` +
//! `broadcast::Sender<()>` under the assumption that signals don't cross
//! VDoms. That assumption turned out to be wrong in 0.7 — verified in Phase 4
//! where `ghost.set(...)` in a floating window's `WindowEvent::Moved` handler
//! triggers a re-render of `FloatingGhostOverlay` + `use_ghost_redock` in the
//! main window on the very next tick. The plan's wrapper would just add lock
//! contention and a second indirection for the same behaviour.
//!
//! This module formalizes the "share a signal to a new window" pattern so the
//! floating-window spawner and any future secondary windows use the same
//! three-context bundle (state + connection + ghost) without drift.

use dioxus::prelude::*;

use super::drag::DropGhost;
use super::DockState;
use crate::hooks::connection::ConnectionState;

/// Convenience bundle of every signal a secondary window needs to render the
/// full dock UI — fetched from the current scope's context once so the
/// spawner can thread them through `with_root_context` in one line each.
#[derive(Copy, Clone)]
pub struct SharedContexts {
    pub state: Signal<DockState>,
    pub conn: Signal<ConnectionState>,
    pub ghost: Signal<Option<DropGhost>>,
}

impl SharedContexts {
    /// Read all three signals from the current scope's context. Call this
    /// only inside a component / hook that has access to the main-window
    /// contexts (i.e., inside `DockRoot` or something rendered below
    /// `use_drop_ghost()` in the App root).
    pub fn from_context() -> Self {
        Self {
            state: use_context::<Signal<DockState>>(),
            conn: use_context::<Signal<ConnectionState>>(),
            ghost: use_context::<Signal<Option<DropGhost>>>(),
        }
    }

    /// Attach every signal in this bundle to a freshly-built VirtualDom as
    /// root contexts — Dioxus's supported way to cross the VDom boundary.
    /// The returned VDom's `use_context::<Signal<DockState>>()` etc. will
    /// resolve to the same signals as the parent window.
    pub fn inject(self, dom: VirtualDom) -> VirtualDom {
        dom.with_root_context(self.state)
            .with_root_context(self.conn)
            .with_root_context(self.ghost)
    }
}
