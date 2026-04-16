use std::time::Instant;

use dioxus::prelude::*;
use uuid::Uuid;

use super::commands::DropSide;
use super::model::{BindingId, DockNode, DockState};
use super::path::DockPath;

/// Global drag state for a tab being moved.
///
/// While `Some`, a viewport-wide overlay captures mouse events and renders
/// the drop-zone preview. A tab's `onmousedown` seeds this signal; the
/// overlay's `onmouseup` commits the drop and clears it.
#[derive(Clone, Debug, PartialEq)]
pub struct DragState {
    pub binding: BindingId,
    /// Human-readable label for the floating chip at the cursor.
    pub label: String,
    /// Cursor position in client (viewport) coordinates.
    pub cursor: (f64, f64),
    /// Current hovered target + side, recomputed on each mousemove.
    pub hover: Option<Hover>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hover {
    pub leaf_path: DockPath,
    /// Leaf rect in client coordinates: (x, y, w, h).
    pub rect: (f64, f64, f64, f64),
    pub side: DropSide,
}

/// Compute which drop zone of a `rect` the cursor `(cx, cy)` is over.
/// Ported from rc-dock's "edge mode": normalize distances against a
/// 500 px cap so the 30% cutoff behaves the same on huge panels.
///
/// Returns `Center` when the cursor is in the middle region (≥ 30% from
/// every edge) and no edge zone wins.
pub fn hit_side(rect: (f64, f64, f64, f64), cursor: (f64, f64)) -> DropSide {
    let (rx, ry, rw, rh) = rect;
    let (cx, cy) = cursor;
    let wr = rw.min(500.0).max(1.0);
    let hr = rh.min(500.0).max(1.0);
    let left = (cx - rx) / wr;
    let right = (rx + rw - cx) / wr;
    let top = (cy - ry) / hr;
    let bottom = (ry + rh - cy) / hr;

    let min = left.min(right).min(top).min(bottom);
    if min >= 0.30 {
        return DropSide::Center;
    }
    if min == left {
        DropSide::Left
    } else if min == right {
        DropSide::Right
    } else if min == top {
        DropSide::Top
    } else {
        DropSide::Bottom
    }
}

/// Compute the client-space rectangle to render as the drop preview for
/// `side` inside `rect`. Center = full rect; edges = the leading or trailing
/// 50 % in the relevant dimension.
pub fn preview_rect(rect: (f64, f64, f64, f64), side: DropSide) -> (f64, f64, f64, f64) {
    let (x, y, w, h) = rect;
    match side {
        DropSide::Center => (x, y, w, h),
        DropSide::Left => (x, y, w * 0.5, h),
        DropSide::Right => (x + w * 0.5, y, w * 0.5, h),
        DropSide::Top => (x, y, w, h * 0.5),
        DropSide::Bottom => (x, y + h * 0.5, w, h * 0.5),
    }
}

/// Walk `main_tree` and collect (path, leaf_bindings) for every Leaf.
/// Used by the overlay to iterate candidates when doing hit testing.
pub fn collect_leaf_paths(state: &DockState) -> Vec<DockPath> {
    let mut out = vec![];
    fn walk(node: &DockNode, path: &mut DockPath, out: &mut Vec<DockPath>) {
        match node {
            DockNode::Leaf { .. } => out.push(path.clone()),
            DockNode::Split { first, second, .. } => {
                path.push(0);
                walk(first, path, out);
                path.pop();
                path.push(1);
                walk(second, path, out);
                path.pop();
            }
        }
    }
    let mut path = vec![];
    walk(&state.main_tree, &mut path, &mut out);
    out
}

/// Provide the drag signal via context. Call once at the DockRoot level.
pub fn use_drag_state() -> Signal<Option<DragState>> {
    let signal = use_signal(|| None::<DragState>);
    use_hook(|| provide_context(signal))
}

/// Sent by a `FloatingWindowRoot` while the user is dragging its title bar —
/// read by the main window's DragOverlay to render the re-dock ghost preview.
///
/// Screen coordinates (not viewport): the main window converts to its own
/// client space using its `inner_position()`.
#[derive(Clone, Debug, PartialEq)]
pub struct DropGhost {
    pub window: Uuid,
    pub screen_pos: (f64, f64),
    pub size: (f64, f64),
    pub last_move: Instant,
    /// True while the floating window's drag is active; flipped to false by
    /// the debounce when moves stop for 150 ms. On the falling edge, the
    /// main window commits the re-dock.
    pub dragging: bool,
}

/// Provide the ghost signal at DockRoot; floating windows get it via
/// `with_root_context` on their VirtualDom.
pub fn use_drop_ghost() -> Signal<Option<DropGhost>> {
    let signal = use_signal(|| None::<DropGhost>);
    use_hook(|| provide_context(signal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_when_cursor_is_middle() {
        let rect = (0.0, 0.0, 400.0, 400.0);
        assert_eq!(hit_side(rect, (200.0, 200.0)), DropSide::Center);
    }

    #[test]
    fn left_edge_detected() {
        let rect = (0.0, 0.0, 400.0, 400.0);
        // 30 px from left = 30/400 = 0.075, well below 0.30 threshold.
        assert_eq!(hit_side(rect, (30.0, 200.0)), DropSide::Left);
    }

    #[test]
    fn bottom_edge_on_tall_panel() {
        let rect = (0.0, 0.0, 400.0, 800.0);
        // Cap at 500 → heightRate = 500. bottom distance = 50 / 500 = 0.10.
        assert_eq!(hit_side(rect, (200.0, 750.0)), DropSide::Bottom);
    }

    #[test]
    fn preview_right_takes_right_half() {
        let rect = (0.0, 0.0, 200.0, 100.0);
        assert_eq!(
            preview_rect(rect, DropSide::Right),
            (100.0, 0.0, 100.0, 100.0)
        );
    }
}
