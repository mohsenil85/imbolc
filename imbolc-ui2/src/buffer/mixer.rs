use crate::buffer::{Buffer, BufferId, BufferKind, ModelineInfo};
use crate::state::AppState;
use crate::ui::action_id::ActionId;
use crate::ui::input::InputEvent;
use crate::ui::keymap::Keymap;
use crate::ui::render::{Rect, RenderBuf};
use crate::ui::style::Style;
use imbolc_types::action::Action;

/// Mixer overview buffer (port of MixerPane).
/// MVP: shows track list with levels.
pub struct MixerBuffer {
    #[allow(dead_code)]
    keymap: Keymap,
}

impl MixerBuffer {
    pub fn new() -> Self {
        Self {
            keymap: Keymap::new(),
        }
    }
}

impl Buffer for MixerBuffer {
    fn id(&self) -> BufferId {
        BufferId::global(BufferKind::Mixer)
    }

    fn layer_name(&self) -> &'static str {
        "mixer"
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

        let header = "Mixer";
        let x = area.x + 1;
        let y = area.y;
        buf.draw_str(x, y, header, Style::new().fg(palette.accent).bold());

        for (i, track) in state.tracks.tracks.iter().enumerate() {
            let row = y + 1 + i as u16;
            if row >= area.y + area.height {
                break;
            }
            let db = if track.channel_strip.level <= 0.0 {
                "-inf".to_string()
            } else {
                let db_val = 20.0 * track.channel_strip.level.log10();
                format!("{:+.1}dB", db_val)
            };
            let mute_str = if track.channel_strip.mute { " [M]" } else { "" };
            let solo_str = if track.channel_strip.solo { " [S]" } else { "" };
            let line = format!(
                "  {:2}. {:<16} {:>7}{}{}",
                i + 1,
                &track.name[..track.name.len().min(16)],
                db,
                mute_str,
                solo_str,
            );
            let style = if Some(i) == state.tracks.selected {
                Style::new().fg(palette.fg).bg(palette.selection_bg)
            } else {
                Style::new().fg(palette.fg)
            };
            buf.draw_str(x, row, &line, style);
        }
    }

    fn modeline_info(&self, state: &AppState) -> ModelineInfo {
        ModelineInfo {
            buffer_name: "mixer".to_string(),
            context: format!("{} tracks", state.tracks.tracks.len()),
            mode: String::new(),
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }
}
