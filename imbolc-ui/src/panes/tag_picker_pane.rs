use std::any::Any;

use crate::state::AppState;
use crate::ui::action_id::ActionId;
use crate::ui::input::{MouseButton, MouseEventKind};
use crate::ui::pane::*;
use crate::ui::{InputEvent, Keymap, MouseEvent, Rect, RenderBuf};
use imbolc_types::{AutomationTarget, TagAction};

/// Modal pane for assigning a parameter to a tag.
/// Shown when user presses 'T' in instrument edit pane.
pub struct TagPickerPane {
    keymap: Keymap,
    selected: usize,
    /// Deferred action to dispatch on next tick (for multi-step operations)
    deferred: Option<Action>,
}

impl TagPickerPane {
    pub fn new(keymap: Keymap) -> Self {
        Self {
            keymap,
            selected: 0,
            deferred: None,
        }
    }

    fn tag_count(&self, state: &AppState) -> usize {
        // Tags + "New tag..." option
        state.session.param_tags.tags.len() + 1
    }

    fn confirm_selection(&mut self, state: &AppState) -> Action {
        let pending = state.session.param_tags.pending_target.clone();
        if let Some(target) = pending {
            let tag_count = state.session.param_tags.tags.len();
            if self.selected < tag_count {
                // Add to existing tag
                Action::Tag(TagAction::AddTarget(self.selected, target))
            } else {
                // "New tag..." — create tag now, defer AddTarget to next tick
                let name = default_tag_name(state);
                self.deferred = Some(Action::Tag(TagAction::AddTarget(tag_count, target)));
                Action::Tag(TagAction::CreateTag(name))
            }
        } else {
            Action::Nav(NavAction::PopPane)
        }
    }
}

impl Pane for TagPickerPane {
    fn id(&self) -> &'static str {
        PaneId::TagPicker.as_str()
    }

    fn handle_action(&mut self, action: ActionId, _event: &InputEvent, state: &AppState) -> Action {
        use crate::ui::action_id::AddActionId;
        let count = self.tag_count(state);
        match action {
            ActionId::Add(AddActionId::Next) => {
                if count > 0 {
                    self.selected = (self.selected + 1).min(count - 1);
                }
                Action::None
            }
            ActionId::Add(AddActionId::Prev) => {
                self.selected = self.selected.saturating_sub(1);
                Action::None
            }
            ActionId::Add(AddActionId::Cancel) => Action::Nav(NavAction::PopPane),
            ActionId::Add(AddActionId::Confirm) => self.confirm_selection(state),
            _ => Action::None,
        }
    }

    fn handle_mouse(&mut self, event: &MouseEvent, area: Rect, state: &AppState) -> Action {
        let count = self.tag_count(state);
        let modal = crate::ui::layout_helpers::center_rect(area, 40, (count as u16 + 4).min(20));

        let content_y = modal.y + 2;
        if event.column >= modal.x && event.column < modal.x + modal.width {
            let row = event.row as i32 - content_y as i32;
            if row >= 0 && (row as usize) < count {
                self.selected = row as usize;
                if event.kind == MouseEventKind::Down(MouseButton::Left) {
                    return self.confirm_selection(state);
                }
            }
        }
        Action::None
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        use crate::ui::style::{Color, Style};

        let tags = &state.session.param_tags.tags;
        let count = self.tag_count(state);
        let height = (count as u16 + 4).min(20);
        let modal = crate::ui::layout_helpers::center_rect(area, 40, height);

        // Border + Title
        let border_style = Style::new().fg(Color::new(100, 200, 255));
        buf.draw_block(modal, " Tag Parameter ", border_style, border_style);

        // Pending target info
        if let Some(ref target) = state.session.param_tags.pending_target {
            let desc = describe_target(target, state);
            let desc_style = Style::new().fg(Color::new(180, 180, 180));
            let truncated: String = desc.chars().take((modal.width - 4) as usize).collect();
            buf.draw_str(modal.x + 2, modal.y + 1, &truncated, desc_style);
        }

        // List tags
        let content_y = modal.y + 2;
        for (i, tag) in tags.iter().enumerate() {
            let y = content_y + i as u16;
            if y >= modal.y + modal.height - 1 {
                break;
            }
            let label = format!("  {}  ({} params)", tag.name, tag.targets.len());
            let style = if i == self.selected {
                Style::new().fg(Color::WHITE).bg(Color::SELECTION_BG)
            } else {
                Style::new().fg(Color::WHITE)
            };
            let truncated: String = label.chars().take((modal.width - 2) as usize).collect();
            buf.draw_str(modal.x + 1, y, &truncated, style);
            let remaining = (modal.width as usize - 2).saturating_sub(truncated.len());
            if remaining > 0 {
                buf.draw_str(
                    modal.x + 1 + truncated.len() as u16,
                    y,
                    &" ".repeat(remaining),
                    style,
                );
            }
        }

        // "New tag..." option
        let new_y = content_y + tags.len() as u16;
        if new_y < modal.y + modal.height - 1 {
            let style = if self.selected == tags.len() {
                Style::new()
                    .fg(Color::new(100, 255, 100))
                    .bg(Color::SELECTION_BG)
            } else {
                Style::new().fg(Color::new(100, 255, 100))
            };
            let label = "  + New tag...";
            buf.draw_str(modal.x + 1, new_y, label, style);
            let remaining = (modal.width as usize - 2).saturating_sub(label.len());
            if remaining > 0 {
                buf.draw_str(
                    modal.x + 1 + label.len() as u16,
                    new_y,
                    &" ".repeat(remaining),
                    style,
                );
            }
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn on_enter(&mut self, _state: &AppState) {
        self.selected = 0;
        self.deferred = None;
    }

    fn tick(&mut self, _state: &AppState) -> Vec<Action> {
        if let Some(action) = self.deferred.take() {
            vec![action]
        } else {
            vec![]
        }
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn default_tag_name(state: &AppState) -> String {
    let n = state.session.param_tags.tags.len() + 1;
    format!("Tag {}", n)
}

fn describe_target(target: &AutomationTarget, state: &AppState) -> String {
    use imbolc_types::{BusParameter, GlobalParameter, InstrumentParameter};
    match target {
        AutomationTarget::Track(id, InstrumentParameter::Standard(param)) => {
            let inst_name = state
                .instruments
                .instrument(*id)
                .map(|i| i.name.as_str())
                .unwrap_or("?");
            format!("{} > {}", inst_name, param.name())
        }
        AutomationTarget::Bus(id, BusParameter::Level) => {
            format!("Bus {} > Level", id.get())
        }
        AutomationTarget::Global(GlobalParameter::Bpm) => "Global > BPM".to_string(),
        AutomationTarget::Global(GlobalParameter::TimeSignature) => "Global > Time Sig".to_string(),
        AutomationTarget::Generative(param) => format!("Gen > {:?}", param),
    }
}
