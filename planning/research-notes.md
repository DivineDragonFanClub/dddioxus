# Research Items

Topics to kick off with subagents before starting Phases 3 and 4.
Do them just-in-time, not all up-front — saves context.

## Phase 3 research (before writing docking code)

### R1. rc-dock tree model

Read the source of [rc-dock](https://github.com/ticlo/rc-dock),
specifically:

- Its `DockLayout` box tree shape.
- The tab-move / split mutation helpers.
- How it handles empty-leaf collapse after moves.
- Drop-zone hit testing (how 5-zone regions are computed per leaf).

Goal: we're not porting it, but the API shape + the collapse logic
are the parts most likely to have subtle bugs if we invent them.

### R2. Drop-zone visuals

Good visual feedback during drag is the UX bar. Look at:

- **Chrome/Firefox devtools** tab-drag visuals.
- **VS Code** drag-to-split.
- **Unity** inspector docking.

Report: what color / opacity / animation is used, and how the zone
boundaries are visually indicated (border highlight vs filled rect vs
dashed outline).

### R3. Dioxus `DockCommand` pattern

Search Dioxus community patterns for command-style state mutations.
Alternatives: direct signal writes, reducer pattern, event bus. Pick
one and document.

## Phase 4 research (before writing floating-window code)

### R4. Dioxus `new_window()` concrete API

Look at `dioxus_desktop::use_window()` + `.new_window(cfg, root)`:

- What exactly does `cfg: Config` accept? (`with_decorations(false)`
  in particular.)
- What does the second arg look like for passing a specific root
  component with props?
- How does context flow? Does the new window's VirtualDom inherit
  context from the spawning window, or does it need its own
  `provide_context` setup?
- Are hot-reload and devtools available in secondary windows?

Path to check: `/Users/doge/workspace/dioxus/packages/desktop/src/hooks.rs`,
`/Users/doge/workspace/dioxus/examples/08-apis/window_popup.rs`.

### R5. Tao window events and drag

Look at `tao::event::WindowEvent` variants:

- `Moved(PhysicalPosition)` — how frequently fires during drag?
- `Focused(bool)` — reliable for detecting drag-end on macOS?
  Windows? Linux?
- `tao::window::Window::drag_window()` — does it block until drag
  ends, or return immediately? What's the return value?

Cross-reference with tao's GitHub issues for known bugs.

### R6. Cross-window message passing

`dioxus_desktop` exposes a `DesktopService` with window proxies.
Research:

- Can one window send a typed event to another window's event loop?
- Alternative: `tokio::sync::broadcast` as a shared runtime resource.
- Best pattern for a small, in-process cross-window state.

### R7. Custom title bar on WKWebView (macOS)

For Phase 4's custom title bar on floating windows:

- Does `with_decorations(false)` on macOS give us a fully blank
  window with no title bar at all?
- Does our custom CSS-drawn bar interact with macOS's traffic-light
  buttons? (Typically we hide decorations entirely and draw our own
  close/minimize.)
- Is there a way to keep native traffic lights but hide the title
  area? (`titlebarAppearsTransparent`?)

Path to check: `/Users/doge/workspace/dioxus/packages/desktop/src/launch.rs`
and tao's macOS window configuration code.

## Pre-implementation decisions to revisit

None currently outstanding — all items in [decisions.md](decisions.md)
are locked. If research turns up a blocker (e.g., `drag_window()`
doesn't work on macOS), raise it immediately before proceeding.
