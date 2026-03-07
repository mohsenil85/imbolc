use crate::buffer::{Buffer, BufferId, BufferKind, ModelineInfo};
use crate::state::AppState;
use crate::ui::action_id::ActionId;
use crate::ui::input::InputEvent;
use crate::ui::keymap::Keymap;
use crate::ui::render::{Rect, RenderBuf};
use crate::ui::style::Style;
use imbolc_types::action::Action;
use imbolc_types::{SourceTypeExt, TrackId};

/// Track parameter editor buffer (port of TrackEditPane).
/// MVP: shows track name and basic info.
#[allow(dead_code)]
pub struct TrackEditBuffer {
    track_id: TrackId,
    keymap: Keymap,
}

#[allow(dead_code)]
impl TrackEditBuffer {
    pub fn new(track_id: TrackId) -> Self {
        Self {
            track_id,
            keymap: Keymap::new(),
        }
    }
}

impl Buffer for TrackEditBuffer {
    fn id(&self) -> BufferId {
        BufferId::scoped(BufferKind::TrackEdit, self.track_id)
    }

    fn layer_name(&self) -> &'static str {
        "instrument_edit"
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

        let track = state.tracks.tracks.iter().find(|t| t.id == self.track_id);
        let header = match track {
            Some(t) => format!(
                "Track: {} ({})",
                t.name,
                t.source.display_name(&state.session.custom_synthdefs)
            ),
            None => "Track not found".to_string(),
        };
        let x = area.x + 1;
        let y = area.y;
        buf.draw_str(x, y, &header, Style::new().fg(palette.fg));
    }

    fn modeline_info(&self, state: &AppState) -> ModelineInfo {
        let name = state
            .tracks
            .tracks
            .iter()
            .find(|t| t.id == self.track_id)
            .map(|t| t.name.clone())
            .unwrap_or_else(|| "?".to_string());
        ModelineInfo {
            buffer_name: format!("track-edit:{}", self.track_id),
            context: name,
            mode: String::new(),
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn is_valid(&self, state: &AppState) -> bool {
        state.tracks.tracks.iter().any(|t| t.id == self.track_id)
    }
}
