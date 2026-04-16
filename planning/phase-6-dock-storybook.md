# Phase 6 (follow-up) — Dock Storybook

**Branch:** continue on `dock/phase-5-cross-window-state`
**Budget:** ~1 hour
**Visible change:** a new `/dev/dock-workspace` route that mounts `DockRoot`
with fake contexts so the user can exercise docking mechanics without a
live game connection.

## Why

Phases 3–5 landed drag-to-split, tab-close, eject-to-floating, and
drag-back re-docking — but all of these mechanics live inside
`DockRoot`, which is only reachable from the `/` route behind
`ConnectionProvider`. Without a running Cobalt_dev Skyline mod, the
user never sees `DockRoot`, so none of the dock work is testable.

## Goal

Ship one storybook page that renders `DockRoot` end-to-end with
enough context plumbing that dragging, splitting, resizing, ejecting,
and redocking all work. The inner panel bodies (Scene tree /
Transform inspector / etc.) will show their disconnected error
states — that's acceptable; this story is a **docking sandbox**, not
a panel sandbox.

## Scope

In scope:
- New route `/dev/dock-workspace` → `DevDockWorkspace` component.
- Story-local `Signal<DockState>` (starts from `DockState::default_layout()`).
- Story-local `Signal<ConnectionState>` (always `Disconnected`).
- Story-local `Signal<Option<DropGhost>>`.
- Mounts `DockRoot {}` in the standard dev layout.
- Small toolbar above with "Reset layout" + "Open {Scene,Globals,Procs,Inspector,Another Inspector}" buttons so the user can build up a layout with extra tabs to drag around.
- Sidebar entry under a new "Simulations" group (alongside the existing "Scene viewer" simulation).

Out of scope:
- Stubbing the inner panel RPC calls. Accept the error states.
- Stories for `LeafView` / `SplitView` / `InspectorFrame` in isolation — `DockRoot` already exercises them.
- Fake fixtures for transform / components data.

## State isolation

The App's `use_dock_state()` already provides a `Signal<DockState>` at
the top of every route. If the story provides its own signal at its
root scope, `use_context::<Signal<DockState>>()` inside `DockRoot`
resolves to whichever is closest — **the story's**. So mutations made
in the story don't touch `~/.dddioxus_layout.json`, and the real
`DockRoot` on `/` is unaffected.

Same pattern for `ConnectionState` and `DropGhost`: story provides
fresh signals, descendants pick them up.

Verify with a quick `dock` read inside `DevDockWorkspace`:
provide-then-read should see the story's signal, not the App's.

## What doesn't work (documented in the story)

- Panels show "Loading..." → error once their fetch fails. The story
  renders a brief banner up top explaining this: "Inner panels are
  disconnected — this page tests docking mechanics only."
- Persistence writes are not blocked, but they land on the story's
  local signal, which isn't observed by the autosaver hook. No disk
  IO from within the story.
- The story's `DockRoot` still shares the OS window with the rest of
  the app, so ejected floats open real OS windows. That's the point —
  you can test the eject/redock flow.

## Risks

1. **Floating spawner double-picks-up on route switch**: when the user
   navigates from `/dev/dock-workspace` to another route, the story's
   scope drops but its `use_floating_spawner` may have already
   dispatched floating windows. Those windows survive the navigation
   because they're OS-level. They'd then try to read the (now-dropped)
   story state via `with_root_context` and render empty/errored
   content. Mitigation: on unmount of `DevDockWorkspace`, explicitly
   drain `state.floating` via `CloseFloating` for each. Not critical
   for v1 — if it's annoying the user can click × on each float.

2. **Bounds accumulation**: the story's state is re-created on every
   remount, but previously-ejected floats know nothing about this.
   Accept for v1.

## Acceptance criteria

- [ ] Navigate to `/dev/dock-workspace` → see Scene + Inspector split.
- [ ] Drag the Scene tab onto the right edge of the Inspector leaf
      → creates a new vertical split.
- [ ] Drag the splitter in either direction → resize works.
- [ ] Click × on a locked tab → closes it, leaf collapses.
- [ ] Click the story's "Open Inspector" button → new inspector tab
      appears in whichever leaf holds inspectors.
- [ ] Drag a tab past the viewport edge → floating window appears.
- [ ] Drag the floating window's title bar back over the main pane →
      drop-zone preview renders.
- [ ] Release over a valid zone → float closes, content re-docks.
- [ ] Click the story's "Reset layout" → original split reinstated.
- [ ] Navigate away → the real `/` route is unaffected (its
      `~/.dddioxus_layout.json` isn't touched by the story).

## Implementation plan

Single commit, one new file + three small edits:

1. **New** `src/dev/stories/dock_workspace.rs`:
   - `DevDockWorkspace` component.
   - Story-local signals for state / conn / ghost with
     `use_hook(|| provide_context(...))`.
   - Renders a small toolbar (reset + 4 open-panel buttons) then
     `DockRoot {}`.
2. **Edit** `src/dev/stories/mod.rs`: `pub mod dock_workspace;` +
   `pub use dock_workspace::DevDockWorkspace;`.
3. **Edit** `src/main.rs` `Route`: add
   `#[route("/dev/dock-workspace")] DevDockWorkspace {}`.
4. **Edit** `src/dev/mod.rs` `DevSidebar`:
   - Add `Route::DevDockWorkspace {}` to `is_dev_route`.
   - New "Dock workspace" entry under the existing "Simulations"
     header.

No other files change. No tests change. Commit message:
"Add dock-workspace simulation story (follow-up to Phase 5)".
