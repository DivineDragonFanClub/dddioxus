# Phase 4 — Ejectable Floating Windows with Drag-Back Re-docking

**Branch:** `dock/phase-4-floating-windows`
**Budget:** ~1.5–2 days
**Visible change:** drag a tab past the edge of the main window to
spawn a new OS window for that panel. Drag the floating window's
title bar back over the main window to re-dock.

## Prerequisites

Spawn research subagents on the topics in
[research-notes.md](research-notes.md) under "Phase 4 research"
**before** writing a line of code for this phase.

## Goal

Multi-window inspection. A user on a dual-monitor rig can drag
inspectors onto the second screen; everything keeps working.

## Ejection

When the user drags a tab past the main window's viewport edge and
releases:

1. Spawn a new Dioxus window via `use_window().new_window(Config::new().with_window(...), FloatingWindowRoot)`.
2. Add a `FloatingWindow` entry to `DockState.floating` containing a
   single-leaf tree with the dragged binding.
3. Remove the binding from the source main-window leaf (cleanup as
   in Phase 3).

The new window has `id: Uuid` which the `FloatingWindowRoot`
component uses to identify its slice of state.

## FloatingWindowRoot

```rust
#[component]
pub fn FloatingWindowRoot(window_id: Uuid) -> Element {
    // Find the matching FloatingWindow in DockState.
    // Render its tree with the same DockNodeView used in main.
}
```

Uses the shared `DockState` from Phase 5's refactored state. Until
Phase 5 lands, this can be wired with `Arc<Mutex<DockState>>` as a
stopgap provided via context in both windows.

## Custom title bar

The floating window's title bar will be **custom-drawn** so we can
hook into it for drag-back detection. The `Config` used for the
new window sets `with_decorations(false)` and the top of
`FloatingWindowRoot` renders a custom bar:

```rust
div {
    "data-component": "FloatingTitleBar",
    style: "-webkit-app-region: drag",  // macOS: lets OS drag the window
    // Or: custom onmousedown that calls tao's drag_window()
    // ... title label, close button ...
}
```

Two paths:
- **CSS `-webkit-app-region: drag`** (macOS / WKWebView): works, but
  we don't get programmatic mouse events during drag, so we can't
  render a live ghost in the main window.
- **Custom JS drag + tao `drag_window()`**: `onmousedown` on the title
  bar calls `window.drag_window()` which kicks the OS drag. We also
  listen to `WindowEvent::Moved` events on the floating window and
  broadcast them to the main window for ghost rendering.

**Recommended: the second path.** It enables full drag-back UX.
See research-notes.md for deeper dive.

## Drag-back re-docking (option C)

While a floating window is being dragged:

1. Its `FloatingWindowRoot` intercepts `WindowEvent::Moved` events
   and broadcasts current screen bounds to all other windows (via
   the broadcast channel set up in Phase 5).
2. The main window's `DockRoot` subscribes; when the floating
   window's rect overlaps the main window's viewport:
   - Compute which leaf the cursor is over (using screen-to-viewport
     coordinate conversion with `tao::window::Window::inner_position()`).
   - Show a ghost preview — same drop-zone overlay as Phase 3.
3. On drag end (detected by a debounce on `Moved` events, or cleanly
   via `drag_window()` returning), if the cursor is over a valid
   drop zone:
   - Close the floating window.
   - Move its bindings into the main tree per the drop zone.

## Window-drag-end detection

The hard part. Options:

- `WindowEvent::Focused(true)` — fires when drag completes, usually
  reliable on macOS.
- Debounce `Moved` events — if no `Moved` for ~150 ms, assume drag
  ended. Works but has lag.
- `drag_window()` returning — only works if we manually initiate
  drag via our custom title bar + tao call. Clean and clear.

**Plan: use `drag_window()` + custom title bar.** Fallback to
`Focused` if that proves flaky on some platform.

## State sync

Floating windows need to see the same `DockState` and
`ConnectionState` as the main window. Phase 5 formalizes this.
For Phase 4, bootstrap with:

```rust
// Main window creates an Arc<RwLock<DockState>> and a broadcast channel.
// When spawning a floating window, pass these via the window's root
// component props.
```

This is not ideal and will be replaced in Phase 5.

## Ejection trigger

In Phase 3's drag-drop flow, when a tab's drag ends **outside** the
main window's viewport:

- Create a new `FloatingWindow` with the binding's tree.
- Spawn the new OS window.
- Remove from source leaf.

Viewport check: the overlay's onmouseup fires with client
coordinates. If `x < 0 || x > window_width || y < 0 || y > window_height`
→ eject. Use a small margin (~10 px) to avoid accidental ejection.

## Acceptance criteria

- [ ] Drag a tab off the main window → new OS window appears with
      that panel.
- [ ] The floating window works identically to a docked panel —
      drag values, copy/paste, refresh, etc.
- [ ] Drag the floating window's custom title bar over the main
      window → ghost preview appears in the main window.
- [ ] Release on a valid drop zone → floating window closes, content
      re-docks in the main window.
- [ ] Close the floating window via × → binding is preserved as a
      locked inspector in the main window's default location.
- [ ] Floating window state persists across restarts (bounds saved
      per window in `DockState.floating[*].bounds`).

## Risks

- **Platform-specific drag behaviours**: test on macOS primarily,
  but confirm tao API at least compiles for Windows/Linux.
- **WKWebView limitations**: custom title bar may feel different
  from native; tune CSS.
- **Two windows racing on state**: Phase 5's single-writer pattern
  fixes this properly; until then, be aware of potential race
  conditions in the stopgap Arc<Mutex>.

## Commit plan

1. `Config::with_decorations(false)` + custom title bar component.
2. Spawn floating window via `new_window()`; binding moves into
   `DockState.floating`.
3. Eject trigger on drag-outside-viewport.
4. Floating window closes on ×; binding returns to main.
5. `WindowEvent::Moved` listener + broadcast stub.
6. Main window renders ghost preview on overlap.
7. Drag-end detection + re-dock commit.
8. Bounds persistence.
