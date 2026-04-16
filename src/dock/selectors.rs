use super::model::{Binding, BindingId, DockState, PanelKind};

/// Returns the single inspector that currently follows selection, if any.
pub fn follow_inspector(state: &DockState) -> Option<&Binding> {
    state
        .bindings
        .values()
        .find(|b| b.follows_selection && matches!(b.kind, PanelKind::Inspector))
}

/// Convenience: path of the follow-selection inspector.
pub fn follow_inspector_path(state: &DockState) -> Option<String> {
    follow_inspector(state).and_then(|b| b.path.clone())
}

/// Update the follow-selection inspector's path. Creates one if none exists yet
/// (shouldn't happen with `default_layout`, but we stay defensive).
pub fn set_follow_inspector_path(state: &mut DockState, path: Option<String>) {
    let mut found = false;
    for b in state.bindings.values_mut() {
        if b.follows_selection && matches!(b.kind, PanelKind::Inspector) {
            b.path = path.clone();
            found = true;
        }
    }
    if !found {
        let binding = Binding {
            id: BindingId::new(),
            kind: PanelKind::Inspector,
            path,
            follows_selection: true,
            title: None,
        };
        state.bindings.insert(binding.id, binding);
    }
}
