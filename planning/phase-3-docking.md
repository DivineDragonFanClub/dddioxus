# Phase 3 — Splits, Drag-Drop, and Single-Route Restructure

**Branch:** `dock/phase-3-docking`
**Budget:** ~1.5 days
**Visible change:** full dockable UI. Drag tabs to split panels,
resize splits, open Scene/Globals/Procs as dockable leaves. The
sidebar's Scene / Globals / Procs become "open panel" buttons rather
than route links.

## Prerequisites

Before starting this phase, **spawn research subagents** on the
topics listed in [research-notes.md](research-notes.md) under
"Phase 3 research".

## Goal

Replace the current flex-row layout with a recursive `DockRoot` that
walks the `DockNode` tree. Enable drag-drop to reorganize leaves and
create splits.

## Route restructure

`main.rs`:

```rust
#[derive(Clone, Routable, Debug, PartialEq)]
pub enum Route {
    #[layout(Shell)]
        #[route("/")]
        Workspace {},       // replaces Scene / Globals / Procs
    #[end_layout]

    #[cfg(any(debug_assertions, feature = "dev"))]
    #[layout(DevShell)]    // not really a layout; the dev routes stay
        ...
}

#[component]
fn Workspace() -> Element {
    rsx! {
        ConnectionProvider {
            DockRoot {}
        }
    }
}
```

Old `Scene`, `Globals`, `Procs` components deleted.

## Sidebar behaviour change

`shell.rs`'s main-app Sidebar (non-dev):

- "Scene" / "Globals" / "Procs" become buttons that either focus
  the existing leaf of that type or open a new one.
- Add a "Reset layout" button at the bottom.

```rust
fn open_panel(state: &mut DockState, kind: PanelKind) {
    // If a binding of this kind already exists, focus it.
    // Otherwise, add a new binding + append to a sensible default leaf.
}
```

## New components

```
src/dock/
  view.rs        — DockRoot + DockNodeView (recursive)
  leaf.rs        — LeafView: tab strip + active-binding body
  split.rs       — SplitView: two children with resizable splitter
  drag.rs        — drag state signal, drop-zone overlay, mutation helpers
  commands.rs    — DockCommand enum + apply_command(state, cmd)
```

Use `DockCommand`s (as an enum) rather than directly mutating
`DockState` from UI handlers. Easier to log, undo later, test.

```rust
pub enum DockCommand {
    MoveBindingToLeaf { binding: BindingId, target: LeafId, position: usize },
    SplitLeaf { leaf: LeafId, binding: BindingId, side: Side, new_ratio: f32 },
    ResizeSplit { split: SplitId, new_ratio: f32 },
    CloseBinding { binding: BindingId },
    OpenPanel { kind: PanelKind },
    ResetLayout,
}
```

`LeafId` / `SplitId` need a way to address tree nodes. Options: use a
path like `Vec<usize>` (walk from root), or assign a `Uuid` to every
DockNode. Path-based is simpler and trees are small; use path.

## Drag-drop UX

Based on the pattern in **rc-dock** and **Floating UI** docs:

1. **Drag start**: `onmousedown` on a tab. Capture the binding id and
   the tab's bounding rect.
2. **Drag active**: render a viewport-wide fixed overlay (z-50+) that
   captures mousemove. For each visible leaf, compute 5 drop zones:
   - Top edge (top 20% of leaf) — split horizontal, new leaf on top
   - Bottom edge (bottom 20%) — split horizontal, new leaf on bottom
   - Left edge (left 20%) — split vertical, new leaf on left
   - Right edge (right 20%) — split vertical, new leaf on right
   - Center — add to the leaf's bindings (tab)
3. **Hover highlight**: under the cursor, render a semi-transparent
   rectangle showing where the dropped content will land.
4. **Drop**: on mouseup, emit a `DockCommand` (SplitLeaf or
   MoveBindingToLeaf). `apply_command` mutates `DockState`.
5. **Leaf cleanup**: if a source leaf is emptied by the move, walk
   up and collapse: `Split { first: empty_leaf, second: X } → X`.

## Split resizing

Between the two children of any `Split`, render a 1–2px draggable
splitter. Reuse the proven pattern from
`components/procs_view.rs` (onmousedown → drag_state → overlay with
onmousemove computing new ratio → onmouseup clears). Move the shared
pattern into `src/dock/splitter.rs` as a small helper component so
it's not copy-pasted.

## Leaf rendering (tabs)

Each `Leaf` renders:

- A tab strip (one tab per binding).
- The active binding's body below.
- A "+" button at the end of the strip to open a new panel.
- Tabs draggable (starts the drag-drop flow above).

## Panel body dispatch

Each `Binding.kind` maps to a component:

```rust
match binding.kind {
    PanelKind::Scene      => rsx! { ScenePanel { /* read state, wire handlers */ } },
    PanelKind::Globals    => rsx! { GlobalsPanel { ... } },
    PanelKind::Procs      => rsx! { ProcsPanel { ... } },
    PanelKind::Inspector  => rsx! { InspectorBody { binding_id: ... } },
}
```

Existing panels already take props and are pure — they just need
their container components (`SceneView`, `GlobalsView`, `ProcsView`,
`TransformInspector`, `ComponentsPanel`) to be invoked here. Those
still handle fetch/rpc.

## Acceptance criteria

- [ ] Single `/` route renders a dockable workspace.
- [ ] Default layout shows Scene on left, Inspector on right.
- [ ] Drag a tab to an edge of any leaf → creates a new split.
- [ ] Drag a tab to center → adds to that leaf's tab strip.
- [ ] Resize splits by dragging split borders.
- [ ] Sidebar buttons open new panels.
- [ ] Layout saved and restored across app restarts.
- [ ] Reset Layout menu item restores the default.
- [ ] `/dev` UI Storybook still works unaffected.

## Risks

- **Drag-drop math**: getting drop zones + split ratios right takes
  iteration. Start with exact-20% zones and a 50/50 default split ratio;
  tune after testing.
- **Collapsing empty leaves**: edge cases when the root leaf is the
  one being emptied. Rule: the root always contains at least one
  leaf; if the last binding is closed, reset to default layout or
  show an empty-state.
- **Performance**: recursive render of a small tree is fine. Don't
  over-memoize.

## Commit plan

1. Route collapse to `/` + Workspace component (no dock yet, just
   swap).
2. `DockNode` recursive rendering (Splits + Leaves) with static
   default layout, no drag-drop yet.
3. Resizable splits.
4. Sidebar "open panel" buttons.
5. Drag-drop: tab drag, overlay, drop zones.
6. Drag-drop: commit commands + cleanup.
7. Reset Layout menu item.
8. Polish pass.
