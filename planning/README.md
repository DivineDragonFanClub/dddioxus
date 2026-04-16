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
- Each branch starts from `master`. Open a PR, merge to master before
  starting the next phase. Always keep master runnable.
- Commit between substeps within a phase — don't let a branch grow a
  single giant commit.

## Total budget

~6–7 focused days end-to-end. Real calendar time 2–3 weeks with
review cycles.

## Current state (as of writing this plan)

Working branch: `vec3-improvements` (4 commits ahead of master) with
the drag-to-edit Vec3Editor, storybook, manual connect, command-ID
fix, Scene simulation, etc. All of that should be merged to master
before Phase 0 begins — Phase 0 branches from a clean post-merge
master.
