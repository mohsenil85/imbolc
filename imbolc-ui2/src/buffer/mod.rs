pub mod mixer;
pub mod scratch;
pub mod track_edit;

use std::collections::HashMap;

use crate::state::AppState;
use crate::ui::action_id::ActionId;
use crate::ui::input::{InputEvent, MouseEvent};
use crate::ui::keymap::Keymap;
use crate::ui::render::{Rect, RenderBuf};
use imbolc_types::action::Action;
use imbolc_types::TrackId;

/// Identifies the kind of buffer content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum BufferKind {
    Scratch,
    TrackEdit,
    Mixer,
}

impl BufferKind {
    pub fn display_name(self) -> &'static str {
        match self {
            BufferKind::Scratch => "scratch",
            BufferKind::TrackEdit => "track-edit",
            BufferKind::Mixer => "mixer",
        }
    }
}

/// Unique identifier for a buffer instance.
/// Track-scoped buffers use `scope` to distinguish e.g. `track-edit:3` from `track-edit:7`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BufferId {
    pub kind: BufferKind,
    pub scope: Option<TrackId>,
}

#[allow(dead_code)]
impl BufferId {
    pub fn global(kind: BufferKind) -> Self {
        Self { kind, scope: None }
    }

    pub fn scoped(kind: BufferKind, track_id: TrackId) -> Self {
        Self {
            kind,
            scope: Some(track_id),
        }
    }

    pub fn display_name(&self) -> String {
        match self.scope {
            Some(id) => format!("{}:{}", self.kind.display_name(), id),
            None => self.kind.display_name().to_string(),
        }
    }
}

/// Information displayed in a window's modeline.
pub struct ModelineInfo {
    pub buffer_name: String,
    pub context: String,
    pub mode: String,
}

/// A content view that can be displayed in a window.
#[allow(dead_code)]
pub trait Buffer {
    fn id(&self) -> BufferId;
    fn layer_name(&self) -> &'static str;
    fn handle_action(&mut self, action: ActionId, event: &InputEvent, state: &AppState) -> Action;
    fn handle_raw_input(&mut self, _event: &InputEvent, _state: &AppState) -> Action {
        Action::None
    }
    fn handle_mouse(&mut self, _event: &MouseEvent, _area: Rect, _state: &AppState) -> Action {
        Action::None
    }
    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState);
    fn modeline_info(&self, state: &AppState) -> ModelineInfo;
    fn keymap(&self) -> &Keymap;
    fn on_focus(&mut self, _state: &AppState) {}
    fn on_blur(&mut self, _state: &AppState) {}
    fn tick(&mut self, _state: &AppState) -> Vec<Action> {
        vec![]
    }
    fn is_valid(&self, _state: &AppState) -> bool {
        true
    }
}

/// Registry of all buffer instances. Buffers persist even when not visible.
#[allow(dead_code)]
pub struct BufferRegistry {
    buffers: HashMap<BufferId, Box<dyn Buffer>>,
}

#[allow(dead_code)]
impl BufferRegistry {
    pub fn new() -> Self {
        Self {
            buffers: HashMap::new(),
        }
    }

    pub fn register(&mut self, buffer: Box<dyn Buffer>) {
        let id = buffer.id();
        self.buffers.insert(id, buffer);
    }

    pub fn get(&self, id: &BufferId) -> Option<&dyn Buffer> {
        self.buffers.get(id).map(|b| &**b)
    }

    pub fn get_mut(&mut self, id: &BufferId) -> Option<&mut dyn Buffer> {
        match self.buffers.get_mut(id) {
            Some(b) => Some(&mut **b),
            None => None,
        }
    }

    /// Remove buffers whose track no longer exists.
    /// Returns buffer IDs that were removed.
    pub fn gc_invalid(&mut self, state: &AppState) -> Vec<BufferId> {
        let invalid: Vec<BufferId> = self
            .buffers
            .iter()
            .filter(|(_, b)| !b.is_valid(state))
            .map(|(id, _)| *id)
            .collect();
        for id in &invalid {
            self.buffers.remove(id);
        }
        invalid
    }

    pub fn buffer_ids(&self) -> Vec<BufferId> {
        self.buffers.keys().copied().collect()
    }
}
