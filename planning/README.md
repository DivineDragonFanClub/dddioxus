# Docking + Floating Inspectors — Planning

Plan for implementing a Unity-style docking layout with per-inspector
lock, drag-to-split, and ejectable-with-drag-back floating windows.

## Read order for a fresh session

1. [decisions.md](decisions.md) — everything the user has already
   locked in. **Re-read before questioning any of these.**
2. [phase-0-data-model.md](phase-0-data-model.md) — start here.
3. [phase-1-lock-stack.md](phase-1-lock-stack.md)
4. [phase-3-docking.md](phase-3-docking.md) *(Phase 2 is intentionally
   skipped — see decisions.md)*
5. [phase-4-floating-windows.md](phase-4-floating-windows.md)
6. [phase-5-cross-window-state.md](phase-5-cross-window-state.md)
7. [research-notes.md](research-notes.md) — topics to research with
   subagents before Phases 3 and 4.

## Workflow

- One branch per phase: `dock/phase-0-data-model`, `dock/phase-1-lock-stack`,
  `dock/phase-3-docking`, `dock/phase-4-floating-windows`,
  `dock/phase-5-cross-window-state`.
- **`vec3-improvements` is NOT being merged to master yet.** Phase 0
  branches off `vec3-improvements`. Each subsequent phase branches
  off the previous one (chain), so the whole docking line of work
  stacks on top of `vec3-improvements` until the user says otherwise.
- Commit between substeps within a phase — don't let a branch grow a
  single giant commit.

## Total budget

~6–7 focused days end-to-end. Real calendar time 2–3 weeks with
review cycles.

## Current state (as of writing this plan)

Working branch: `vec3-improvements`, ahead of master with the
drag-to-edit Vec3Editor, storybook, manual connect, command-ID fix,
Scene simulation, data-component labels, etc. **Not merged to master
yet** per user direction — Phase 0 branches directly off
`vec3-improvements`.

## Tooling: dioxus-inspector is live

The `dioxus-inspector` HTTP bridge is wired up in `src/main.rs::App`
behind `cfg(any(debug_assertions, feature = "dev"))`. When running
`dx serve` or a debug build, it listens on `http://127.0.0.1:9999`
with these endpoints:

| Endpoint | Use it for |
|---|---|
| `GET /dom` | Read the live DOM tree without asking the user for a screenshot |
| `POST /query` | Query by CSS selector — `[data-component="Inspector"]` etc. |
| `POST /validate-classes` | Check if a Tailwind class actually exists in the prebuilt CSS (would have saved us multiple rounds on `cursor-ew-resize`) |
| `POST /inspect` | Visibility / clip analysis on a specific element |
| `GET /diagnose` | UI health check |
| `POST /screenshot` | macOS-only window capture |
| `POST /eval` | Run arbitrary JS in the webview |

**If you're inspecting the running app during a work session**, prefer
these endpoints over asking the user for screenshots. Register the
`dioxus-inspector` MCP server in Claude Code's config (pointing at
`http://127.0.0.1:9999`) to get tool access. The user can confirm
when the bridge is running.
