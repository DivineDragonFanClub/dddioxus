use super::model::{Binding, BindingId, DockNode, DockState, PanelKind};

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

/// Update the follow-selection inspector's path. Creates one and appends it to
/// the main tree's first leaf if none exists yet (shouldn't happen with
/// `default_layout`, but we stay defensive).
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
        let id = binding.id;
        state.bindings.insert(id, binding);
        append_to_first_leaf(&mut state.main_tree, id);
    }
}

/// Returns all inspector bindings, in insertion order where possible.
/// HashMap iteration order is unstable — for UI stacking the caller should
/// sort, e.g. by id.0 as a stable tiebreaker.
pub fn inspector_bindings(state: &DockState) -> Vec<Binding> {
    let mut out: Vec<Binding> = state
        .bindings
        .values()
        .filter(|b| matches!(b.kind, PanelKind::Inspector))
        .cloned()
        .collect();
    // Follow-selection inspector first, then locked ones ordered by id for stability.
    out.sort_by(|a, b| match (a.follows_selection, b.follows_selection) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.id.0.cmp(&b.id.0),
    });
    out
}

/// Clone the follow-selection inspector's current path into a new locked
/// inspector. Returns the new binding id. No-op if no follow inspector exists.
pub fn pin_follow_inspector(state: &mut DockState) -> Option<BindingId> {
    let src = follow_inspector(state)?;
    let pinned = Binding {
        id: BindingId::new(),
        kind: PanelKind::Inspector,
        path: src.path.clone(),
        follows_selection: false,
        title: src.path.clone(),
    };
    let id = pinned.id;
    state.bindings.insert(id, pinned);
    append_to_first_leaf(&mut state.main_tree, id);
    Some(id)
}

/// Turn `follows_selection` on for exactly one inspector, off for all others.
pub fn set_follow_flag_exclusive(state: &mut DockState, target: BindingId) {
    for (id, b) in state.bindings.iter_mut() {
        if matches!(b.kind, PanelKind::Inspector) {
            b.follows_selection = *id == target;
        }
    }
}

/// Remove a binding from the HashMap and from every Leaf that references it,
/// in `main_tree` and in every floating window. Collapsing empty leaves is
/// deferred to Phase 3 (the stack UI in Phase 1 renders from the HashMap
/// directly so a dangling Leaf here is harmless).
pub fn remove_binding(state: &mut DockState, id: BindingId) {
    state.bindings.remove(&id);
    strip_from_tree(&mut state.main_tree, id);
    for fw in state.floating.iter_mut() {
        strip_from_tree(&mut fw.tree, id);
    }
}

fn strip_from_tree(node: &mut DockNode, id: BindingId) {
    match node {
        DockNode::Leaf { bindings, active } => {
            bindings.retain(|b| *b != id);
            if *active == Some(id) {
                *active = bindings.first().copied();
            }
        }
        DockNode::Split { first, second, .. } => {
            strip_from_tree(first, id);
            strip_from_tree(second, id);
        }
    }
}

fn append_to_first_leaf(node: &mut DockNode, id: BindingId) {
    match node {
        DockNode::Leaf { bindings, active } => {
            bindings.push(id);
            if active.is_none() {
                *active = Some(id);
            }
        }
        DockNode::Split { first, .. } => append_to_first_leaf(first, id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pin_creates_locked_clone() {
        let mut state = DockState::default_layout();
        set_follow_inspector_path(&mut state, Some("/Foo".into()));
        let pinned = pin_follow_inspector(&mut state).expect("has follow");

        let b = &state.bindings[&pinned];
        assert!(!b.follows_selection);
        assert_eq!(b.path.as_deref(), Some("/Foo"));
        assert_eq!(
            state
                .bindings
                .values()
                .filter(|b| b.follows_selection)
                .count(),
            1
        );
    }

    #[test]
    fn exclusive_follow_flips_others() {
        let mut state = DockState::default_layout();
        let pinned = pin_follow_inspector(&mut state).unwrap();
        set_follow_flag_exclusive(&mut state, pinned);

        assert!(state.bindings[&pinned].follows_selection);
        assert_eq!(
            state
                .bindings
                .values()
                .filter(|b| b.follows_selection)
                .count(),
            1
        );
    }

    #[test]
    fn remove_binding_cleans_leaves() {
        let mut state = DockState::default_layout();
        let pinned = pin_follow_inspector(&mut state).unwrap();
        remove_binding(&mut state, pinned);

        assert!(!state.bindings.contains_key(&pinned));
        fn leaf_contains(node: &DockNode, id: BindingId) -> bool {
            match node {
                DockNode::Leaf { bindings, .. } => bindings.contains(&id),
                DockNode::Split { first, second, .. } => {
                    leaf_contains(first, id) || leaf_contains(second, id)
                }
            }
        }
        assert!(!leaf_contains(&state.main_tree, pinned));
    }
}
