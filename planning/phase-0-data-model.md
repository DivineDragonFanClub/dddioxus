# Phase 0 — Data Model & Persistence

**Branch:** `dock/phase-0-data-model`
**Budget:** ~1 day
**Visible change:** none — app runs identically, but is now backed by
the new data model.

## Goal

Stand up the `DockState` type, its persistence, and provide it via
context at the App root. Refactor the existing `selected_path` signal
in `SceneView` to sync into the new state. Do **not** change any UI
yet.

## New files

```
src/dock/
  mod.rs         — re-exports + provide_context helper
  model.rs       — types (DockState, DockNode, InspectorBinding, etc.)
  persistence.rs — load_from_disk / save_to_disk (debounced)
```

## Types

```rust
// src/dock/model.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;  // add `uuid = { version = "1", features = ["v4", "serde"] }`

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BindingId(pub Uuid);

impl BindingId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum PanelKind {
    Inspector,  // the default — bound to a scene-object path
    Scene,
    Globals,
    Procs,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Binding {
    pub id: BindingId,
    pub kind: PanelKind,
    /// Inspector-only: path to the bound GameObject. None = no binding yet.
    pub path: Option<String>,
    /// Inspector-only: if true, updates to match selected_path.
    /// Exactly one inspector across all windows should have this true.
    pub follows_selection: bool,
    /// Optional user-visible label. Falls back to kind + path.
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Axis { Horizontal, Vertical }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DockNode {
    Leaf {
        bindings: Vec<BindingId>,
        /// Active tab within this leaf. Must be in `bindings` or None.
        active: Option<BindingId>,
    },
    Split {
        dir: Axis,
        /// Ratio of space the first child occupies [0.0, 1.0].
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FloatingWindow {
    pub id: Uuid,
    pub tree: DockNode,
    /// Last known position on screen, for persistence.
    pub bounds: Option<(f64, f64, f64, f64)>,  // x, y, w, h
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DockState {
    pub bindings: HashMap<BindingId, Binding>,
    pub main_tree: DockNode,
    pub floating: Vec<FloatingWindow>,
}

impl DockState {
    /// Default layout: one scene-tree leaf on the left, one inspector
    /// leaf on the right, split 60/40.
    pub fn default_layout() -> Self {
        let scene = Binding { id: BindingId::new(), kind: PanelKind::Scene, path: None, follows_selection: false, title: None };
        let inspector = Binding { id: BindingId::new(), kind: PanelKind::Inspector, path: None, follows_selection: true, title: None };
        let mut bindings = HashMap::new();
        let (sid, iid) = (scene.id, inspector.id);
        bindings.insert(sid, scene);
        bindings.insert(iid, inspector);
        Self {
            bindings,
            main_tree: DockNode::Split {
                dir: Axis::Horizontal,
                ratio: 0.6,
                first: Box::new(DockNode::Leaf { bindings: vec![sid], active: Some(sid) }),
                second: Box::new(DockNode::Leaf { bindings: vec![iid], active: Some(iid) }),
            },
            floating: vec![],
        }
    }
}
```

## Persistence

```rust
// src/dock/persistence.rs
use std::path::PathBuf;

fn layout_path() -> Option<PathBuf> {
    std::env::var("HOME").ok().map(|h| PathBuf::from(h).join(".dddioxus_layout.json"))
}

pub fn load() -> Option<DockState> {
    let path = layout_path()?;
    let s = std::fs::read_to_string(&path).ok()?;
    serde_json::from_str(&s).ok()
}

pub fn save(state: &DockState) {
    if let Some(path) = layout_path() {
        if let Ok(s) = serde_json::to_string_pretty(state) {
            let _ = std::fs::write(&path, s);
        }
    }
}
```

Debounce writes by 250 ms. Implementation sketch: a `use_effect` in
the App root that watches the `DockState` signal and spawns a
`tokio::time::sleep` before saving, cancelling prior pending saves.

## App wiring

In `main.rs`:

```rust
#[component]
fn App() -> Element {
    use_connection();

    let initial = dock::persistence::load().unwrap_or_else(DockState::default_layout);
    let state = use_signal(|| initial);
    use_hook(|| provide_context(state));

    // Debounced save on change
    use_effect(move || {
        let snapshot = state.read().clone();
        spawn(async move {
            tokio::time::sleep(Duration::from_millis(250)).await;
            tokio::task::spawn_blocking(move || dock::persistence::save(&snapshot));
        });
    });

    // ... existing rsx ...
}
```

## SceneView sync

Remove the local `selected_path: Signal<Option<String>>` in
`SceneView`. Replace with reading/writing the follow-selection
inspector's `path` in `DockState.bindings`:

```rust
// pseudocode
let state = use_context::<Signal<DockState>>();
let selected = state.read().bindings.values()
    .find(|b| b.follows_selection && matches!(b.kind, PanelKind::Inspector))
    .and_then(|b| b.path.clone());

// on tree-node click:
state.write().bindings.values_mut()
    .filter(|b| b.follows_selection && matches!(b.kind, PanelKind::Inspector))
    .for_each(|b| b.path = Some(clicked_path.clone()));
```

A helper module `src/dock/selectors.rs` with getters/setters like
`follow_inspector_path(state)` and `set_follow_inspector_path(state, p)`
would clean this up.

## Acceptance criteria

- [ ] New `src/dock/` module compiles with no warnings.
- [ ] `DockState::default_layout()` serializes/deserializes to JSON
      successfully (roundtrip test or at least manual via `dbg!`).
- [ ] App starts up with `DockState` in context; existing
      Scene/Globals/Procs views still work identically.
- [ ] `selected_path` state in `SceneView` now lives in
      `DockState` (single source of truth).
- [ ] `~/.dddioxus_layout.json` is written when the user clicks
      around; read on startup (verify by killing the app and
      relaunching).

## Out of scope

- Actually rendering the tree — Phase 3 does that.
- Any UI for lock / pin / close — Phase 1.
- Multiple inspectors — Phase 1.
- Floating windows — Phase 4.

## Commit plan

Suggested commits within this phase:
1. Add `uuid` dep + create `src/dock/model.rs` with types.
2. Add `src/dock/persistence.rs`.
3. Wire `DockState` into App context.
4. Refactor `SceneView` to read/write `follow_inspector_path` from state.
5. Commit.

Open PR `dock/phase-0-data-model` → master.
