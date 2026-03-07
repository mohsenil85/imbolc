use crate::buffer::{Buffer, BufferId, BufferKind, ModelineInfo};
use crate::state::AppState;
use crate::ui::action_id::ActionId;
use crate::ui::input::InputEvent;
use crate::ui::keymap::Keymap;
use crate::ui::render::{Rect, RenderBuf};
use crate::ui::style::Style;
use imbolc_types::action::Action;

/// A minimal empty buffer showing centered "imbolc" text.
pub struct ScratchBuffer {
    #[allow(dead_code)]
    keymap: Keymap,
}

impl ScratchBuffer {
    pub fn new() -> Self {
        Self {
            keymap: Keymap::new(),
        }
    }
}

impl Buffer for ScratchBuffer {
    fn id(&self) -> BufferId {
        BufferId::global(BufferKind::Scratch)
    }

    fn layer_name(&self) -> &'static str {
        "scratch"
    }

    fn handle_action(
        &mut self,
        _action: ActionId,
        _event: &InputEvent,
        _state: &AppState,
    ) -> Action {
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let palette = crate::ui::style::Palette::from(&state.session.theme);
        buf.fill_bg(area, palette.bg);

        let text = "imbolc";
        let x = area.x + area.width.saturating_sub(text.len() as u16) / 2;
        let y = area.y + area.height / 2;
        buf.draw_str(x, y, text, Style::new().fg(palette.dim));

        let subtitle = "buffer/window UI";
        let sx = area.x + area.width.saturating_sub(subtitle.len() as u16) / 2;
        buf.draw_str(sx, y + 1, subtitle, Style::new().fg(palette.dim));
    }

    fn modeline_info(&self, _state: &AppState) -> ModelineInfo {
        ModelineInfo {
            buffer_name: "scratch".to_string(),
            context: String::new(),
            mode: String::new(),
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }
}
