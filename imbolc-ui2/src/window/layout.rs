use super::{Direction, WindowTree};
use crate::buffer::{BufferId, BufferKind};

/// Named layout presets for common window arrangements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum Layout {
    /// Single buffer fills the screen.
    Single,
    /// Two buffers side by side (horizontal split).
    TwoColumn,
}

#[allow(dead_code)]
impl Layout {
    /// Apply this layout to a window tree, starting from the given buffer.
    pub fn apply(&self, tree: &mut WindowTree, initial_buffer: BufferId) {
        // Reset to single window with the initial buffer
        tree.close_others();
        tree.switch_buffer(initial_buffer);

        match self {
            Layout::Single => {}
            Layout::TwoColumn => {
                tree.split(Direction::Horizontal);
                // Second window gets scratch by default
                tree.focus_next();
                tree.switch_buffer(BufferId::global(BufferKind::Scratch));
                tree.focus_prev();
            }
        }
    }
}
