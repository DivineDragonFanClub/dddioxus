# Phase 5 — Cross-window Reactive State

**Branch:** `dock/phase-5-cross-window-state`
**Budget:** ~1 day
**Visible change:** state changes in any window (main or floating)
propagate to all other windows immediately. No more stale reads.

## Goal

Formalize multi-window state sharing so `DockState`, `ConnectionState`,
and the `Vec3` clipboard are all single sources of truth shared
across every open Dioxus window.

## The problem Phase 4 leaves behind

Each Dioxus window has its own `VirtualDom`. Signals don't cross that
boundary. Phase 4's stopgap — an `Arc<Mutex<DockState>>` passed to
each window's root — works for reads but doesn't trigger re-renders
in the OTHER windows when one window writes.

## The pattern

**Single writer + broadcast wake signal.**

```rust
// src/state.rs (new)
pub struct SharedDockState {
    inner: Arc<RwLock<DockState>>,
    tx: broadcast::Sender<()>,
}

impl SharedDockState {
    pub fn read(&self) -> DockStateReadGuard {
        DockStateReadGuard(self.inner.read().unwrap())
    }

    pub fn mutate<F>(&self, f: F) where F: FnOnce(&mut DockState) {
        {
            let mut w = self.inner.write().unwrap();
            f(&mut w);
        }
        let _ = self.tx.send(());
    }
}
```

Each window's root:

```rust
#[component]
fn WindowRoot() -> Element {
    let shared = use_context::<SharedDockState>();
    // Bump a local version signal whenever the shared state changes,
    // forcing re-render.
    let mut version = use_signal(|| 0u64);
    use_future(move || {
        let mut rx = shared.tx.subscribe();
        async move {
            while rx.recv().await.is_ok() {
                version.write(); // triggers re-render by write
                *version.write() += 1;
            }
        }
    });

    let state = shared.read();
    // Render based on state. Use `state.main_tree` or floating[*] as appropriate.
    ...
}
```

All mutations in any window go through `shared.mutate(|s| { ... })`,
which locks, applies, unlocks, and broadcasts. Every other window's
`use_future` wakes up, bumps its local version, and re-renders
reading the fresh shared state.

## Refactoring path

1. Replace `Signal<DockState>` with `SharedDockState` in the App
   context.
2. Every `state.read()` callsite stays similar; pattern is still
   "read + render".
3. Every `state.write()` callsite changes to
   `state.mutate(|s| { ... })`.
4. `ConnectionState` gets the same treatment (`SharedConnectionState`).
5. `Signal<Option<Vec3>>` clipboard stays in-process but if users
   open the Copy/Paste menu in a floating window, the context must
   be available there too — provide via the same shared pattern.
6. Persistence (`dock::persistence::save`) now subscribes to the
   same broadcast; debounced saves happen regardless of which window
   triggered the mutation.

## Actually reading values

Inside components, replace:

```rust
let tree = state.read().main_tree.clone();
```

with:

```rust
let tree = {
    let guard = shared.read();
    guard.0.main_tree.clone()  // clone to drop the lock
};
```

Cloning is fine for our tree sizes. We lock briefly, clone, render.

## Acceptance criteria

- [ ] Opening the same Inspector binding in both main and floating
      window, dragging a Vec3 in one, value updates in the other
      in real time.
- [ ] Copying a Vec3 in main and pasting in floating works.
- [ ] Connection state change visible immediately in both windows.
- [ ] No deadlocks from mutual locking (scope locks tightly).
- [ ] Persistence still works: final state saved correctly after
      any window's mutation.

## Risks

- **Re-render cascade**: every broadcast wakes every window. If 5
  windows are open, one mutation triggers 5 re-renders. Should be
  fine for a debugging tool, but monitor.
- **Lock contention**: keep the critical section in `mutate` tight
  (just the state change, no I/O).
- **Dropped broadcasts**: `tokio::sync::broadcast` channels drop
  messages when slow consumers lag. For us this is fine — a
  dropped wake just means slightly delayed re-render once the next
  broadcast arrives. But consider bumping channel capacity from the
  default 16.

## Commit plan

1. New `src/state.rs` with `SharedDockState` + `SharedConnectionState`.
2. Migrate App and all consumers from `Signal<DockState>` to
   `SharedDockState`.
3. Wire the broadcast subscription in every window root.
4. Clipboard migration.
5. Update persistence to subscribe to the shared broadcast.
