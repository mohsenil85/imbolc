use crate::buffer::ModelineInfo;
use crate::ui::render::{Rect, RenderBuf};
use crate::ui::style::{Palette, Style};

/// Render a per-window modeline (1 row at the bottom of each window).
pub fn render_modeline(
    buf: &mut RenderBuf,
    area: Rect,
    info: &ModelineInfo,
    is_focused: bool,
    palette: &Palette,
) {
    let bg = if is_focused {
        palette.accent
    } else {
        palette.border
    };
    let fg = if is_focused { palette.bg } else { palette.dim };

    // Fill the modeline row
    let style = Style::new().fg(fg).bg(bg);
    for x in area.x..area.x + area.width {
        buf.set_cell(x, area.y, ' ', style);
    }

    // Buffer name (left-aligned)
    let name = format!(" {} ", info.buffer_name);
    buf.draw_str(area.x, area.y, &name, style.bold());

    // Context (after buffer name)
    if !info.context.is_empty() {
        let cx = area.x + name.len() as u16;
        let ctx = format!("  {} ", info.context);
        buf.draw_str(cx, area.y, &ctx, style);
    }

    // Mode (right-aligned)
    if !info.mode.is_empty() {
        let mode_text = format!(" {} ", info.mode);
        let mx = area.x + area.width.saturating_sub(mode_text.len() as u16);
        buf.draw_str(mx, area.y, &mode_text, style);
    }
}
