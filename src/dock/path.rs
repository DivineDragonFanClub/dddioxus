use super::model::{DockNode, DockState};

/// Path from the root of a dock tree down to a specific node. Each entry is
/// the child index at that level: `0` for `first`, `1` for `second` in a
/// `Split`. An empty `DockPath` refers to the root itself.
pub type DockPath = Vec<usize>;

pub fn node_at<'a>(root: &'a DockNode, path: &[usize]) -> Option<&'a DockNode> {
    let mut node = root;
    for &step in path {
        node = match node {
            DockNode::Split { first, second, .. } => match step {
                0 => first,
                1 => second,
                _ => return None,
            },
            _ => return None,
        };
    }
    Some(node)
}

pub fn node_at_mut<'a>(root: &'a mut DockNode, path: &[usize]) -> Option<&'a mut DockNode> {
    let mut node = root;
    for &step in path {
        node = match node {
            DockNode::Split { first, second, .. } => match step {
                0 => first.as_mut(),
                1 => second.as_mut(),
                _ => return None,
            },
            _ => return None,
        };
    }
    Some(node)
}

/// Replace the subtree at `path` inside `root` with `replacement`. When
/// `path` is empty, this replaces `root` itself. Returns `true` on success.
pub fn replace_at(root: &mut DockNode, path: &[usize], replacement: DockNode) -> bool {
    if path.is_empty() {
        *root = replacement;
        return true;
    }
    let Some(parent) = node_at_mut(root, &path[..path.len() - 1]) else {
        return false;
    };
    let last = *path.last().unwrap();
    match parent {
        DockNode::Split { first, second, .. } => match last {
            0 => {
                *first = Box::new(replacement);
                true
            }
            1 => {
                *second = Box::new(replacement);
                true
            }
            _ => false,
        },
        _ => false,
    }
}

/// Post-order sweep that removes empty leaves by lifting the surviving sibling
/// up into the parent Split's slot. Returns `None` when the entire subtree is
/// empty (caller decides whether to replace with a default).
///
/// Ported from rc-dock's `fixBoxData` — see planning/research notes.
pub fn collapse(node: DockNode) -> Option<DockNode> {
    match node {
        DockNode::Leaf { bindings, active } => {
            if bindings.is_empty() {
                None
            } else {
                let active = match active {
                    Some(id) if bindings.contains(&id) => Some(id),
                    _ => bindings.first().copied(),
                };
                Some(DockNode::Leaf { bindings, active })
            }
        }
        DockNode::Split {
            dir,
            ratio,
            first,
            second,
        } => {
            let f = collapse(*first);
            let s = collapse(*second);
            match (f, s) {
                (None, None) => None,
                (Some(only), None) | (None, Some(only)) => Some(only),
                (Some(a), Some(b)) => Some(DockNode::Split {
                    dir,
                    ratio: ratio.clamp(0.05, 0.95),
                    first: Box::new(a),
                    second: Box::new(b),
                }),
            }
        }
    }
}

/// Collapse every tree held in the state. If the main tree collapses to
/// nothing, reset it to an empty placeholder leaf so we never hold an
/// impossible "no root" state.
pub fn collapse_state(state: &mut DockState) {
    state.main_tree = collapse(std::mem::replace(
        &mut state.main_tree,
        DockNode::Leaf {
            bindings: vec![],
            active: None,
        },
    ))
    .unwrap_or(DockNode::Leaf {
        bindings: vec![],
        active: None,
    });

    state.floating.retain_mut(|fw| {
        let collapsed = collapse(std::mem::replace(
            &mut fw.tree,
            DockNode::Leaf {
                bindings: vec![],
                active: None,
            },
        ));
        match collapsed {
            Some(tree) => {
                fw.tree = tree;
                true
            }
            None => false,
        }
    });
}

#[cfg(test)]
mod tests {
    use super::super::model::{Axis, BindingId, DockNode};
    use super::*;

    fn leaf(n: usize) -> DockNode {
        let ids: Vec<BindingId> = (0..n).map(|_| BindingId::new()).collect();
        let active = ids.first().copied();
        DockNode::Leaf {
            bindings: ids,
            active,
        }
    }

    #[test]
    fn collapse_removes_empty_leaf() {
        let tree = DockNode::Split {
            dir: Axis::Horizontal,
            ratio: 0.5,
            first: Box::new(DockNode::Leaf {
                bindings: vec![],
                active: None,
            }),
            second: Box::new(leaf(2)),
        };
        let collapsed = collapse(tree).unwrap();
        assert!(matches!(collapsed, DockNode::Leaf { ref bindings, .. } if bindings.len() == 2));
    }

    #[test]
    fn collapse_keeps_balanced_tree() {
        let tree = DockNode::Split {
            dir: Axis::Vertical,
            ratio: 0.4,
            first: Box::new(leaf(1)),
            second: Box::new(leaf(1)),
        };
        assert!(matches!(collapse(tree), Some(DockNode::Split { .. })));
    }

    #[test]
    fn replace_at_targets_deep_node() {
        let mut tree = DockNode::Split {
            dir: Axis::Horizontal,
            ratio: 0.5,
            first: Box::new(leaf(1)),
            second: Box::new(DockNode::Split {
                dir: Axis::Vertical,
                ratio: 0.5,
                first: Box::new(leaf(1)),
                second: Box::new(leaf(1)),
            }),
        };
        let new_leaf = leaf(5);
        let new_len = match &new_leaf {
            DockNode::Leaf { bindings, .. } => bindings.len(),
            _ => unreachable!(),
        };
        assert!(replace_at(&mut tree, &[1, 0], new_leaf));
        match node_at(&tree, &[1, 0]).unwrap() {
            DockNode::Leaf { bindings, .. } => assert_eq!(bindings.len(), new_len),
            _ => panic!("expected leaf"),
        }
    }
}
