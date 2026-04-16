use super::model::{Axis, Binding, BindingId, DockNode, DockState, PanelKind};
use super::path::{self, DockPath};
use super::selectors;

/// Which edge of a leaf the dropped binding lands on. `Center` means "append
/// to this leaf's tab strip."
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DropSide {
    Top,
    Right,
    Bottom,
    Left,
    Center,
}

impl DropSide {
    pub fn is_edge(self) -> bool {
        !matches!(self, DropSide::Center)
    }

    pub fn axis(self) -> Option<Axis> {
        match self {
            DropSide::Top | DropSide::Bottom => Some(Axis::Vertical),
            DropSide::Left | DropSide::Right => Some(Axis::Horizontal),
            DropSide::Center => None,
        }
    }

    /// In the resulting Split, does the new (dropped) leaf go first?
    /// `Top`/`Left` → yes (new is above/left of existing).
    pub fn new_goes_first(self) -> bool {
        matches!(self, DropSide::Top | DropSide::Left)
    }
}

/// Mutations to the dock tree. All UI drag/drop code emits these rather than
/// mutating the state directly — this makes drag semantics trivial to
/// unit-test and keeps an obvious place to hang undo/logging later.
#[derive(Clone, Debug, PartialEq)]
pub enum DockCommand {
    /// Move `binding` from wherever it currently lives into the `target` leaf,
    /// landing on `side`. Center = tab strip; edge = new split.
    Drop {
        binding: BindingId,
        target: DockPath,
        side: DropSide,
    },
    /// User dragged a Split's divider; set its ratio.
    ResizeSplit { split: DockPath, ratio: f32 },
    /// Remove `binding` from the state and every leaf that references it.
    /// Collapse follows automatically.
    CloseBinding { binding: BindingId },
    /// Open a panel of the given kind: focus an existing binding of that kind
    /// if one exists, otherwise create + append.
    OpenPanel { kind: PanelKind },
    /// Throw away the current layout and reinstantiate defaults.
    ResetLayout,
}

pub fn apply(state: &mut DockState, cmd: DockCommand) {
    match cmd {
        DockCommand::Drop {
            binding,
            target,
            side,
        } => apply_drop(state, binding, &target, side),
        DockCommand::ResizeSplit { split, ratio } => apply_resize(state, &split, ratio),
        DockCommand::CloseBinding { binding } => {
            selectors::remove_binding(state, binding);
        }
        DockCommand::OpenPanel { kind } => apply_open_panel(state, kind),
        DockCommand::ResetLayout => {
            *state = DockState::default_layout();
        }
    }

    path::collapse_state(state);
}

fn apply_drop(state: &mut DockState, binding: BindingId, target: &[usize], side: DropSide) {
    // Snapshot the binding's current leaves so we can remove it after placing.
    // This trivially handles the "drop on the same leaf" case — center drop
    // becomes a reorder, edge drop becomes a split-then-remove-from-original.
    strip_binding_from_tree(&mut state.main_tree, binding);
    for fw in state.floating.iter_mut() {
        strip_binding_from_tree(&mut fw.tree, binding);
    }

    let Some(target_node) = path::node_at_mut(&mut state.main_tree, target) else {
        // Target vanished (shouldn't happen with well-formed paths); append to first leaf.
        append_to_first_leaf(&mut state.main_tree, binding);
        return;
    };

    if side == DropSide::Center {
        match target_node {
            DockNode::Leaf { bindings, active } => {
                bindings.push(binding);
                *active = Some(binding);
            }
            // Dropping center on a Split shouldn't happen (drag logic targets leaves),
            // but be defensive: recurse into first leaf.
            split @ DockNode::Split { .. } => append_to_first_leaf(split, binding),
        }
        return;
    }

    // Edge drop: wrap the target node in a Split containing a new single-binding leaf
    // on the requested side.
    let Some(axis) = side.axis() else {
        return;
    };
    let new_first = side.new_goes_first();
    let existing = std::mem::replace(
        target_node,
        DockNode::Leaf {
            bindings: vec![],
            active: None,
        },
    );
    let new_leaf = DockNode::Leaf {
        bindings: vec![binding],
        active: Some(binding),
    };
    let replacement = if new_first {
        DockNode::Split {
            dir: axis,
            ratio: 0.5,
            first: Box::new(new_leaf),
            second: Box::new(existing),
        }
    } else {
        DockNode::Split {
            dir: axis,
            ratio: 0.5,
            first: Box::new(existing),
            second: Box::new(new_leaf),
        }
    };
    *target_node = replacement;
}

fn apply_resize(state: &mut DockState, path: &[usize], ratio: f32) {
    if let Some(DockNode::Split { ratio: r, .. }) = path::node_at_mut(&mut state.main_tree, path) {
        *r = ratio.clamp(0.05, 0.95);
    }
}

fn apply_open_panel(state: &mut DockState, kind: PanelKind) {
    if let Some(existing) = state
        .bindings
        .values()
        .find(|b| b.kind == kind)
        .map(|b| b.id)
    {
        focus_binding(&mut state.main_tree, existing);
        for fw in state.floating.iter_mut() {
            focus_binding(&mut fw.tree, existing);
        }
        return;
    }

    let follows = matches!(kind, PanelKind::Inspector)
        && selectors::follow_inspector(state).is_none();
    let binding = Binding {
        id: BindingId::new(),
        kind,
        path: None,
        follows_selection: follows,
        title: None,
    };
    let id = binding.id;
    state.bindings.insert(id, binding);
    append_to_first_leaf(&mut state.main_tree, id);
}

fn strip_binding_from_tree(node: &mut DockNode, id: BindingId) {
    match node {
        DockNode::Leaf { bindings, active } => {
            bindings.retain(|b| *b != id);
            if *active == Some(id) {
                *active = bindings.first().copied();
            }
        }
        DockNode::Split { first, second, .. } => {
            strip_binding_from_tree(first, id);
            strip_binding_from_tree(second, id);
        }
    }
}

fn append_to_first_leaf(node: &mut DockNode, id: BindingId) {
    match node {
        DockNode::Leaf { bindings, active } => {
            bindings.push(id);
            *active = Some(id);
        }
        DockNode::Split { first, .. } => append_to_first_leaf(first, id),
    }
}

fn focus_binding(node: &mut DockNode, id: BindingId) {
    match node {
        DockNode::Leaf { bindings, active } => {
            if bindings.contains(&id) {
                *active = Some(id);
            }
        }
        DockNode::Split { first, second, .. } => {
            focus_binding(first, id);
            focus_binding(second, id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::model::PanelKind;
    use super::*;

    #[test]
    fn drop_edge_wraps_target_in_split() {
        let mut state = DockState::default_layout();
        // Pin so we have a movable inspector binding.
        let pinned = selectors::pin_follow_inspector(&mut state).unwrap();

        // Drop it on the right of the root (path = []) — wraps root in a horizontal split.
        apply(
            &mut state,
            DockCommand::Drop {
                binding: pinned,
                target: vec![],
                side: DropSide::Right,
            },
        );

        match &state.main_tree {
            DockNode::Split { dir, .. } => assert_eq!(*dir, Axis::Horizontal),
            _ => panic!("expected split"),
        }
    }

    #[test]
    fn resize_clamps_ratio() {
        let mut state = DockState::default_layout();
        apply(
            &mut state,
            DockCommand::ResizeSplit {
                split: vec![],
                ratio: 2.0,
            },
        );
        match &state.main_tree {
            DockNode::Split { ratio, .. } => assert!(*ratio <= 0.95),
            _ => panic!("expected split"),
        }
    }

    #[test]
    fn open_panel_reuses_existing() {
        let mut state = DockState::default_layout();
        let before = state.bindings.len();
        apply(
            &mut state,
            DockCommand::OpenPanel {
                kind: PanelKind::Scene,
            },
        );
        assert_eq!(state.bindings.len(), before); // Scene binding already exists.
    }

    #[test]
    fn open_panel_creates_new_when_missing() {
        let mut state = DockState::default_layout();
        apply(
            &mut state,
            DockCommand::OpenPanel {
                kind: PanelKind::Globals,
            },
        );
        assert!(state
            .bindings
            .values()
            .any(|b| matches!(b.kind, PanelKind::Globals)));
    }

    #[test]
    fn close_collapses_empty_leaves() {
        let mut state = DockState::default_layout();
        let inspector_id = state
            .bindings
            .values()
            .find(|b| b.kind == PanelKind::Inspector)
            .unwrap()
            .id;
        apply(
            &mut state,
            DockCommand::CloseBinding {
                binding: inspector_id,
            },
        );
        // Main tree should have collapsed from Split{scene, inspector} → just scene-leaf.
        assert!(matches!(state.main_tree, DockNode::Leaf { .. }));
    }
}
