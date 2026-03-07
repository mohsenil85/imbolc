pub mod layout;
pub mod render;

use crate::buffer::BufferId;
use crate::ui::render::Rect;

/// Unique identifier for a window in the tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowId(pub u32);

/// Split direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Horizontal, // left | right
    Vertical,   // top / bottom
}

/// A node in the binary split tree.
enum Node {
    Leaf {
        window_id: WindowId,
        buffer_id: BufferId,
    },
    Split {
        direction: Direction,
        ratio: f32,
        first: Box<Node>,
        second: Box<Node>,
    },
}

/// Binary tree of splits managing window layout.
pub struct WindowTree {
    root: Node,
    focus: WindowId,
    next_id: u32,
}

impl WindowTree {
    /// Create a tree with a single window showing the given buffer.
    pub fn new(buffer_id: BufferId) -> Self {
        Self {
            root: Node::Leaf {
                window_id: WindowId(0),
                buffer_id,
            },
            focus: WindowId(0),
            next_id: 1,
        }
    }

    pub fn focused_window(&self) -> WindowId {
        self.focus
    }

    pub fn focused_buffer(&self) -> BufferId {
        self.find_buffer(self.focus)
            .expect("focused window must exist")
    }

    pub fn find_buffer_for(&self, id: WindowId) -> Option<BufferId> {
        Self::find_buffer_in(&self.root, id)
    }

    fn find_buffer(&self, id: WindowId) -> Option<BufferId> {
        Self::find_buffer_in(&self.root, id)
    }

    fn find_buffer_in(node: &Node, id: WindowId) -> Option<BufferId> {
        match node {
            Node::Leaf {
                window_id,
                buffer_id,
            } => {
                if *window_id == id {
                    Some(*buffer_id)
                } else {
                    None
                }
            }
            Node::Split { first, second, .. } => {
                Self::find_buffer_in(first, id).or_else(|| Self::find_buffer_in(second, id))
            }
        }
    }

    /// Split the focused window in the given direction.
    /// The new window gets a copy of the focused buffer.
    pub fn split(&mut self, direction: Direction) {
        let new_id = WindowId(self.next_id);
        self.next_id += 1;
        Self::split_node(&mut self.root, self.focus, direction, new_id);
    }

    fn split_node(node: &mut Node, target: WindowId, direction: Direction, new_id: WindowId) {
        match node {
            Node::Leaf {
                window_id,
                buffer_id,
            } => {
                if *window_id == target {
                    let buf = *buffer_id;
                    let old_id = *window_id;
                    *node = Node::Split {
                        direction,
                        ratio: 0.5,
                        first: Box::new(Node::Leaf {
                            window_id: old_id,
                            buffer_id: buf,
                        }),
                        second: Box::new(Node::Leaf {
                            window_id: new_id,
                            buffer_id: buf,
                        }),
                    };
                }
            }
            Node::Split { first, second, .. } => {
                Self::split_node(first, target, direction, new_id);
                Self::split_node(second, target, direction, new_id);
            }
        }
    }

    /// Close the focused window. Returns false if it's the last window.
    pub fn close_focused(&mut self) -> bool {
        if self.window_count() <= 1 {
            return false;
        }
        let removed = Self::remove_node(&mut self.root, self.focus);
        if removed {
            // Focus the first leaf
            self.focus = self.first_window_id();
        }
        removed
    }

    /// Close all windows except the focused one.
    pub fn close_others(&mut self) {
        let buf = self.focused_buffer();
        let wid = self.focus;
        self.root = Node::Leaf {
            window_id: wid,
            buffer_id: buf,
        };
    }

    fn remove_node(node: &mut Node, target: WindowId) -> bool {
        match node {
            Node::Leaf { .. } => false,
            Node::Split { first, second, .. } => {
                // Check if first child is the target leaf
                if let Node::Leaf { window_id, .. } = first.as_ref() {
                    if *window_id == target {
                        *node = std::mem::replace(
                            second.as_mut(),
                            Node::Leaf {
                                window_id: WindowId(u32::MAX),
                                buffer_id: crate::buffer::BufferId::global(
                                    crate::buffer::BufferKind::Scratch,
                                ),
                            },
                        );
                        return true;
                    }
                }
                // Check if second child is the target leaf
                if let Node::Leaf { window_id, .. } = second.as_ref() {
                    if *window_id == target {
                        *node = std::mem::replace(
                            first.as_mut(),
                            Node::Leaf {
                                window_id: WindowId(u32::MAX),
                                buffer_id: crate::buffer::BufferId::global(
                                    crate::buffer::BufferKind::Scratch,
                                ),
                            },
                        );
                        return true;
                    }
                }
                Self::remove_node(first, target) || Self::remove_node(second, target)
            }
        }
    }

    /// Focus the next window (cycle through leaves).
    pub fn focus_next(&mut self) {
        let ids = self.leaf_ids();
        if let Some(pos) = ids.iter().position(|id| *id == self.focus) {
            self.focus = ids[(pos + 1) % ids.len()];
        }
    }

    /// Focus the previous window.
    pub fn focus_prev(&mut self) {
        let ids = self.leaf_ids();
        if let Some(pos) = ids.iter().position(|id| *id == self.focus) {
            self.focus = ids[(pos + ids.len() - 1) % ids.len()];
        }
    }

    /// Switch the buffer displayed in the focused window.
    #[allow(dead_code)]
    pub fn switch_buffer(&mut self, buffer_id: BufferId) {
        Self::set_buffer(&mut self.root, self.focus, buffer_id);
    }

    #[allow(dead_code)]
    fn set_buffer(node: &mut Node, target: WindowId, new_buffer: BufferId) {
        match node {
            Node::Leaf {
                window_id,
                buffer_id,
            } => {
                if *window_id == target {
                    *buffer_id = new_buffer;
                }
            }
            Node::Split { first, second, .. } => {
                Self::set_buffer(first, target, new_buffer);
                Self::set_buffer(second, target, new_buffer);
            }
        }
    }

    /// Compute the layout: map each leaf to its screen rect.
    pub fn layout(&self, area: Rect) -> Vec<(WindowId, BufferId, Rect)> {
        let mut result = Vec::new();
        Self::layout_node(&self.root, area, &mut result);
        result
    }

    fn layout_node(node: &Node, area: Rect, out: &mut Vec<(WindowId, BufferId, Rect)>) {
        match node {
            Node::Leaf {
                window_id,
                buffer_id,
            } => {
                out.push((*window_id, *buffer_id, area));
            }
            Node::Split {
                direction,
                ratio,
                first,
                second,
            } => {
                let (first_area, second_area) = split_rect(area, *direction, *ratio);
                Self::layout_node(first, first_area, out);
                Self::layout_node(second, second_area, out);
            }
        }
    }

    fn window_count(&self) -> usize {
        Self::count_leaves(&self.root)
    }

    fn count_leaves(node: &Node) -> usize {
        match node {
            Node::Leaf { .. } => 1,
            Node::Split { first, second, .. } => {
                Self::count_leaves(first) + Self::count_leaves(second)
            }
        }
    }

    fn leaf_ids(&self) -> Vec<WindowId> {
        let mut ids = Vec::new();
        Self::collect_leaf_ids(&self.root, &mut ids);
        ids
    }

    fn collect_leaf_ids(node: &Node, out: &mut Vec<WindowId>) {
        match node {
            Node::Leaf { window_id, .. } => out.push(*window_id),
            Node::Split { first, second, .. } => {
                Self::collect_leaf_ids(first, out);
                Self::collect_leaf_ids(second, out);
            }
        }
    }

    fn first_window_id(&self) -> WindowId {
        Self::first_leaf_id(&self.root)
    }

    fn first_leaf_id(node: &Node) -> WindowId {
        match node {
            Node::Leaf { window_id, .. } => *window_id,
            Node::Split { first, .. } => Self::first_leaf_id(first),
        }
    }

    /// Replace orphaned buffer IDs with a fallback.
    #[allow(dead_code)]
    pub fn replace_buffer(&mut self, old: BufferId, new: BufferId) {
        Self::replace_buffer_node(&mut self.root, old, new);
    }

    #[allow(dead_code)]
    fn replace_buffer_node(node: &mut Node, old: BufferId, new: BufferId) {
        match node {
            Node::Leaf { buffer_id, .. } => {
                if *buffer_id == old {
                    *buffer_id = new;
                }
            }
            Node::Split { first, second, .. } => {
                Self::replace_buffer_node(first, old, new);
                Self::replace_buffer_node(second, old, new);
            }
        }
    }
}

/// Split a rect into two sub-rects along the given direction.
pub fn split_rect(area: Rect, direction: Direction, ratio: f32) -> (Rect, Rect) {
    match direction {
        Direction::Horizontal => {
            let left_w = (area.width as f32 * ratio).round() as u16;
            let right_w = area.width.saturating_sub(left_w);
            (
                Rect::new(area.x, area.y, left_w, area.height),
                Rect::new(area.x + left_w, area.y, right_w, area.height),
            )
        }
        Direction::Vertical => {
            let top_h = (area.height as f32 * ratio).round() as u16;
            let bot_h = area.height.saturating_sub(top_h);
            (
                Rect::new(area.x, area.y, area.width, top_h),
                Rect::new(area.x, area.y + top_h, area.width, bot_h),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::buffer::{BufferId, BufferKind};

    fn scratch() -> BufferId {
        BufferId::global(BufferKind::Scratch)
    }

    fn mixer() -> BufferId {
        BufferId::global(BufferKind::Mixer)
    }

    #[test]
    fn new_tree_has_single_window() {
        let tree = WindowTree::new(scratch());
        let layout = tree.layout(Rect::new(0, 0, 80, 24));
        assert_eq!(layout.len(), 1);
        assert_eq!(layout[0].1, scratch());
    }

    #[test]
    fn split_creates_two_windows() {
        let mut tree = WindowTree::new(scratch());
        tree.split(Direction::Horizontal);
        let layout = tree.layout(Rect::new(0, 0, 80, 24));
        assert_eq!(layout.len(), 2);
        // Both halves should add up to full width
        assert_eq!(layout[0].2.width + layout[1].2.width, 80);
    }

    #[test]
    fn split_vertical_creates_two_rows() {
        let mut tree = WindowTree::new(scratch());
        tree.split(Direction::Vertical);
        let layout = tree.layout(Rect::new(0, 0, 80, 24));
        assert_eq!(layout.len(), 2);
        assert_eq!(layout[0].2.height + layout[1].2.height, 24);
    }

    #[test]
    fn cannot_close_last_window() {
        let mut tree = WindowTree::new(scratch());
        assert!(!tree.close_focused());
    }

    #[test]
    fn close_returns_to_single() {
        let mut tree = WindowTree::new(scratch());
        tree.split(Direction::Horizontal);
        assert_eq!(tree.layout(Rect::new(0, 0, 80, 24)).len(), 2);
        // Focus second window and close it
        tree.focus_next();
        assert!(tree.close_focused());
        assert_eq!(tree.layout(Rect::new(0, 0, 80, 24)).len(), 1);
    }

    #[test]
    fn focus_next_cycles() {
        let mut tree = WindowTree::new(scratch());
        tree.split(Direction::Horizontal);
        let first = tree.focused_window();
        tree.focus_next();
        assert_ne!(tree.focused_window(), first);
        tree.focus_next();
        assert_eq!(tree.focused_window(), first);
    }

    #[test]
    fn switch_buffer_changes_focused() {
        let mut tree = WindowTree::new(scratch());
        tree.switch_buffer(mixer());
        assert_eq!(tree.focused_buffer(), mixer());
    }

    #[test]
    fn close_others_keeps_focused() {
        let mut tree = WindowTree::new(scratch());
        tree.split(Direction::Horizontal);
        tree.split(Direction::Vertical);
        tree.focus_next();
        tree.switch_buffer(mixer());
        let focused = tree.focused_window();
        tree.close_others();
        assert_eq!(tree.focused_window(), focused);
        assert_eq!(tree.focused_buffer(), mixer());
        assert_eq!(tree.layout(Rect::new(0, 0, 80, 24)).len(), 1);
    }

    #[test]
    fn split_rect_horizontal() {
        let area = Rect::new(0, 0, 100, 50);
        let (left, right) = split_rect(area, Direction::Horizontal, 0.5);
        assert_eq!(left.width, 50);
        assert_eq!(right.width, 50);
        assert_eq!(left.x, 0);
        assert_eq!(right.x, 50);
    }

    #[test]
    fn split_rect_vertical() {
        let area = Rect::new(0, 0, 100, 50);
        let (top, bottom) = split_rect(area, Direction::Vertical, 0.5);
        assert_eq!(top.height, 25);
        assert_eq!(bottom.height, 25);
        assert_eq!(top.y, 0);
        assert_eq!(bottom.y, 25);
    }

    #[test]
    fn replace_buffer_updates_all_windows() {
        let mut tree = WindowTree::new(scratch());
        tree.split(Direction::Horizontal);
        // Both windows show scratch
        tree.replace_buffer(scratch(), mixer());
        let layout = tree.layout(Rect::new(0, 0, 80, 24));
        assert!(layout.iter().all(|(_, buf, _)| *buf == mixer()));
    }
}
