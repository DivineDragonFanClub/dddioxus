# Locked-in Decisions

These were discussed and decided in the planning conversation. Do not
re-debate unless the user explicitly raises them.

## Scope

- **All five phases (0, 1, 3, 4, 5) are in scope.** Phase 2 (tabs
  alone) is skipped; tab behaviour is folded into Phase 3 (Leaf has
  a tab strip by virtue of holding multiple bindings).

## Phase 4 re-docking style

- **Option C: full drag-back** — the user can drag a floating
  window's title bar over the main window and release to re-dock.
  Ghost preview shown in the main window during the drag.
- Budget: ~+1 day over the simpler "menu item to re-dock" variant.
- Likely requires custom-drawn title bars on floating windows so we
  can hook into `drag_window()` and get clean "drag ended" events.
  This means floating windows look slightly different from OS-native
  chrome. Acceptable tradeoff.

## Persistence

- **JSON.** Written to `~/.dddioxus_layout.json` (same pattern as
  `~/.dddioxus_last_host`).
- Debounced writes (~250 ms) on every mutation.
- If the file is missing or malformed, fall back to the default
  layout silently.

## Reset to defaults

- Include a **"Reset layout to default"** menu item in the Edit
  submenu (alongside Cut/Copy/Paste/Select All already there).

## Route / layout restructure

- **Collapse `Scene` / `Globals` / `Procs` routes into a single `/`
  route.** The root is a `DockRoot` that renders the current dock
  tree. Scene/Globals/Procs become dockable panel types.
- The main sidebar's Scene / Globals / Procs buttons become
  **"open a panel of this type"** actions (adds a leaf if none
  exists; focuses it if one does).
- Default layout on first run: scene-tree leaf on the left, inspector
  leaf on the right. Globals and Procs hidden until user opens them.
- **`/dev` UI Storybook stays as a separate route** with its own
  non-dock layout. Don't touch DevShell.

## What stays the same

- Connection signal (`Signal<ConnectionState>`) still provided at
  App root. Panels that need the connection (Scene/Globals/Procs)
  read it via context.
- `Vec3Editor`, `DragFloat`, and all drag/format/clipboard behaviour
  is unchanged by this work.
- `~/.dddioxus_last_host` stays for manual-connect persistence.
  Layout persistence is a separate file.

## Research checkpoints

Before starting Phase 3 and Phase 4, spawn research subagents for the
items listed in [research-notes.md](research-notes.md). Don't try to
invent drag-and-drop / multi-window semantics from scratch.
