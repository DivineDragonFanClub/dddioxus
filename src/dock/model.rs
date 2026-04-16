use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub struct BindingId(pub Uuid);

impl BindingId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BindingId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BindingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PanelKind {
    Inspector,
    Scene,
    Globals,
    Procs,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Binding {
    pub id: BindingId,
    pub kind: PanelKind,
    /// Inspector-only: path to the bound GameObject. `None` before first selection.
    pub path: Option<String>,
    /// Inspector-only: true = updates whenever the user clicks a scene node.
    /// Exactly one inspector across the whole workspace should have this true.
    pub follows_selection: bool,
    /// Optional user-visible label. Falls back to kind + path.
    pub title: Option<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum DockNode {
    Leaf {
        bindings: Vec<BindingId>,
        /// Active tab within this leaf. Must appear in `bindings` or be `None`.
        active: Option<BindingId>,
    },
    Split {
        dir: Axis,
        /// Fraction of space the first child occupies, clamped to [0.05, 0.95].
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FloatingWindow {
    pub id: Uuid,
    pub tree: DockNode,
    /// Last known position on screen, for persistence. (x, y, w, h).
    pub bounds: Option<(f64, f64, f64, f64)>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct DockState {
    pub bindings: HashMap<BindingId, Binding>,
    pub main_tree: DockNode,
    pub floating: Vec<FloatingWindow>,
}

impl DockState {
    /// Default layout: scene-tree leaf on the left, inspector leaf on the right, 60/40 split.
    pub fn default_layout() -> Self {
        let scene = Binding {
            id: BindingId::new(),
            kind: PanelKind::Scene,
            path: None,
            follows_selection: false,
            title: None,
        };
        let inspector = Binding {
            id: BindingId::new(),
            kind: PanelKind::Inspector,
            path: None,
            follows_selection: true,
            title: None,
        };
        let (sid, iid) = (scene.id, inspector.id);
        let mut bindings = HashMap::new();
        bindings.insert(sid, scene);
        bindings.insert(iid, inspector);

        Self {
            bindings,
            main_tree: DockNode::Split {
                dir: Axis::Horizontal,
                ratio: 0.6,
                first: Box::new(DockNode::Leaf {
                    bindings: vec![sid],
                    active: Some(sid),
                }),
                second: Box::new(DockNode::Leaf {
                    bindings: vec![iid],
                    active: Some(iid),
                }),
            },
            floating: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_layout_roundtrips_json() {
        let state = DockState::default_layout();
        let json = serde_json::to_string(&state).expect("serialize");
        let back: DockState = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(state, back);
    }

    #[test]
    fn default_layout_has_follow_inspector() {
        let state = DockState::default_layout();
        let follows: Vec<_> = state
            .bindings
            .values()
            .filter(|b| b.follows_selection)
            .collect();
        assert_eq!(follows.len(), 1);
        assert_eq!(follows[0].kind, PanelKind::Inspector);
    }
}
