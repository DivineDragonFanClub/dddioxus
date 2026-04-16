# Phase 1 — Lock + Vertical Stack

**Branch:** `dock/phase-1-lock-stack` *(branches off `dock/phase-0-data-model`; chain continues until the user decides to merge)*
**Budget:** ~half day
**Visible change:** inspector header gains lock/pin/close; multiple
inspectors render stacked in the right column; clicking pin on the
follow-selection inspector clones the current path into a new locked
inspector.

## Goal

First user-visible capability: **view multiple objects at once**. No
layout manipulation yet — all inspectors stack vertically.

## New / changed files

```
src/components/inspector_host.rs  (new)  — renders a stack of inspectors
src/components/inspector.rs       (edit) — gains a header with buttons
src/components/scene_view.rs      (edit) — uses InspectorHost
```

## InspectorHost

Replaces the single-Inspector rendering in `SceneView`. Reads from
`DockState`, filters bindings of kind `Inspector`, renders each
stacked vertically.

```rust
#[component]
pub fn InspectorHost() -> Element {
    let state = use_context::<Signal<DockState>>();
    let bindings: Vec<Binding> = state.read().bindings.values()
        .filter(|b| matches!(b.kind, PanelKind::Inspector))
        .cloned()
        .collect();

    rsx! {
        div {
            "data-component": "InspectorHost",
            class: "flex flex-col shrink-0 bg-gray-900 border-l border-gray-700 overflow-y-auto",
            for b in bindings {
                InspectorFrame {
                    key: "{b.id.0}",
                    binding: b,
                }
            }
        }
    }
}

#[component]
fn InspectorFrame(binding: Binding) -> Element {
    // Header with path label, lock toggle, pin button, close ×.
    // Body: existing Inspector (or empty placeholder if path is None).
}
```

## Inspector header UI

For each binding in the stack, the header shows:

| Control | When visible | Behaviour |
|---|---|---|
| Path label | always | shows `path.unwrap_or("(no selection)")` |
| 🔒 / 🔓 | always | toggles `follows_selection` on this binding. At most one inspector is `follows_selection = true` across all windows — turning it on here flips it off on all others. |
| 📌 Pin | only on the follow-selection inspector | clones the current path into a new locked binding appended to the stack. |
| × Close | only on locked inspectors | removes the binding from the HashMap and from every dock Leaf that references it. |

Use `title` attributes for tooltips on all buttons (same pattern as
DragFloat).

## Selector helpers

`src/dock/selectors.rs` grows:

```rust
pub fn follow_inspector<'a>(state: &'a DockState) -> Option<&'a Binding>;
pub fn pin_follow_inspector(state: &mut DockState) -> BindingId;  // clones follow → new locked
pub fn set_follow_flag(state: &mut DockState, id: BindingId);     // exclusive toggle
pub fn remove_binding(state: &mut DockState, id: BindingId);
```

`remove_binding` also walks `main_tree` + `floating[*].tree` and
removes the id from any `Leaf::bindings` vector; if a Leaf becomes
empty and it's not the root, collapse it (but collapse logic fully
matters in Phase 3 — for Phase 1, since everything renders in a
stack regardless of tree structure, just remove from HashMap).

## SceneView change

SceneView stops rendering `Inspector { path }` directly. Renders
`InspectorHost {}` as the right column instead.

## Acceptance criteria

- [ ] Clicking a scene node updates the follow-selection inspector's
      path (from Phase 0 wiring).
- [ ] Clicking the 📌 pin on the follow-selection inspector creates
      a second inspector below it, locked to the current path.
- [ ] Clicking 🔒 on an inspector toggles its follow flag; at most
      one inspector is following at a time.
- [ ] Clicking × on a locked inspector removes it from the stack.
- [ ] Layout persists across app restarts (Phase 0's save mechanism
      automatically catches these changes).

## Out of scope

- Horizontal splits — Phase 3.
- Tab strip — Phase 3 (as a side-effect of Leaf rendering).
- Pinning to a new OS window — Phase 4.

## Commit plan

1. Selector helpers + exclusive-follow enforcement.
2. `InspectorHost` component + header chrome.
3. Swap SceneView's inspector call for `InspectorHost`.
4. Manual smoke test of the four acceptance criteria.
