use crate::buffer::BufferRegistry;
use crate::chrome::modeline;
use crate::state::AppState;
use crate::ui::render::{Rect, RenderBuf};
use crate::ui::style::Style;
use crate::window::{WindowId, WindowTree};

/// Render all windows in the tree: each buffer's content + its modeline.
pub fn render_window_tree(
    tree: &WindowTree,
    registry: &mut BufferRegistry,
    area: Rect,
    buf: &mut RenderBuf,
    state: &AppState,
) {
    let layout = tree.layout(area);
    let focused = tree.focused_window();
    let palette = crate::ui::style::Palette::from(&state.session.theme);

    for (window_id, buffer_id, window_rect) in &layout {
        // Skip windows that are too small
        if window_rect.height < 3 || window_rect.width < 10 {
            render_too_small(buf, *window_rect, &palette);
            continue;
        }

        // Reserve 1 row at the bottom for the modeline
        let content_rect = Rect::new(
            window_rect.x,
            window_rect.y,
            window_rect.width,
            window_rect.height.saturating_sub(1),
        );
        let modeline_rect = Rect::new(
            window_rect.x,
            window_rect.y + window_rect.height.saturating_sub(1),
            window_rect.width,
            1,
        );

        // Render buffer content
        if let Some(buffer) = registry.get_mut(buffer_id) {
            buffer.render(content_rect, buf, state);

            // Render modeline
            let info = buffer.modeline_info(state);
            let is_focused = *window_id == focused;
            modeline::render_modeline(buf, modeline_rect, &info, is_focused, &palette);
        } else {
            // Buffer not found — fill with error
            buf.fill_bg(content_rect, palette.bg);
            buf.draw_str(
                content_rect.x + 1,
                content_rect.y,
                &format!("buffer not found: {}", buffer_id.display_name()),
                Style::new().fg(palette.error),
            );
        }
    }
}

fn render_too_small(buf: &mut RenderBuf, area: Rect, palette: &crate::ui::style::Palette) {
    buf.fill_bg(area, palette.bg);
    if area.width >= 5 && area.height >= 1 {
        buf.draw_str(area.x, area.y, "small", Style::new().fg(palette.dim));
    }
}

/// Find which window a mouse click landed in.
pub fn hit_test(tree: &WindowTree, area: Rect, column: u16, row: u16) -> Option<WindowId> {
    let layout = tree.layout(area);
    for (window_id, _, rect) in &layout {
        if column >= rect.x
            && column < rect.x + rect.width
            && row >= rect.y
            && row < rect.y + rect.height
        {
            return Some(*window_id);
        }
    }
    None
}
