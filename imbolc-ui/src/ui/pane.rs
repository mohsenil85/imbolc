use std::any::Any;

use super::action_id::ActionId;
use super::style::{Color, Style};
use super::{InputEvent, Keymap, MouseEvent, Rect, RenderBuf};
use crate::state::AppState;

// Re-export all action types from the core crate
pub use crate::action::{
    Action, ArrangementAction, AutomationAction, BusAction, DispatchResult, FileSelectAction,
    GenerativeAction, GroupAction, MasterAction, MixerAction, NavAction, NavIntent, PaneId,
    PianoRollAction, SampleSlicerAction, SequencerAction, ServerAction, SessionAction, StatusEvent,
    ToggleResult, TrackAction, TrackUpdate, VstParamAction,
};

/// Unique pane identity (for PaneManager lookup, logging, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PaneIdStr(pub &'static str);

/// Keybinding layer name (for LayerStack resolution).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerName(pub &'static str);

/// Trait for UI panes (screens/views).
pub trait Pane {
    /// Unique identifier for this pane
    fn id(&self) -> PaneIdStr;

    /// Layer name for keybinding resolution (defaults to id()).
    /// Override when multiple panes share the same keybinding layer.
    fn layer_name(&self) -> LayerName {
        LayerName(self.id().0)
    }

    /// Handle a resolved action ID from the layer system
    fn handle_action(&mut self, action: ActionId, event: &InputEvent, state: &AppState) -> Action;

    /// Handle raw input when layers resolved to Blocked or Unresolved
    fn handle_raw_input(&mut self, _event: &InputEvent, _state: &AppState) -> Action {
        Action::None
    }

    /// Handle mouse input. Area is the full terminal area (same as render receives).
    fn handle_mouse(&mut self, _event: &MouseEvent, _area: Rect, _state: &AppState) -> Action {
        Action::None
    }

    /// Render the pane to the buffer
    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState);

    /// Get the keymap for this pane (for introspection/help)
    fn keymap(&self) -> &Keymap;

    /// Called when this pane becomes active
    fn on_enter(&mut self, _state: &AppState) {}

    /// Called when this pane becomes inactive
    fn on_exit(&mut self, _state: &AppState) {}

    /// Called each frame to check for time-based state changes (e.g., key release).
    /// Returns actions to dispatch (default: empty).
    fn tick(&mut self, _state: &AppState) -> Vec<Action> {
        vec![]
    }

    /// Toggle performance mode (piano/pad keyboard). Returns what happened.
    fn toggle_performance_mode(&mut self, _state: &AppState) -> ToggleResult {
        ToggleResult::NotSupported
    }

    /// Activate piano keyboard on this pane (for cross-pane sync)
    fn activate_piano(&mut self) {}

    /// Activate pad keyboard on this pane
    fn activate_pad(&mut self) {}

    /// Deactivate performance mode (piano/pad) on this pane
    fn deactivate_performance(&mut self) {}

    /// Whether this pane supports piano/pad performance mode.
    /// Panes that return false will auto-exit performance mode on switch.
    fn supports_performance_mode(&self) -> bool {
        false
    }

    /// Return self as Any for downcasting (required for type-specific access)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Manages a stack of panes with one active pane
pub struct PaneManager {
    panes: Vec<Box<dyn Pane>>,
    active_index: usize,
    stack: Vec<usize>,
}

impl PaneManager {
    /// Create a new pane manager with an initial pane
    pub fn new(initial_pane: Box<dyn Pane>) -> Self {
        Self {
            panes: vec![initial_pane],
            active_index: 0,
            stack: Vec::new(),
        }
    }

    /// Add a pane to the manager (does not make it active)
    pub fn add_pane(&mut self, pane: Box<dyn Pane>) {
        self.panes.push(pane);
    }

    /// Get the currently active pane
    pub fn active(&self) -> &dyn Pane {
        self.panes[self.active_index].as_ref()
    }

    /// Get the currently active pane mutably
    pub fn active_mut(&mut self) -> &mut dyn Pane {
        self.panes[self.active_index].as_mut()
    }

    /// Switch to a pane by ID (flat navigation — clears the stack)
    pub fn switch_to(&mut self, id: PaneId, state: &AppState) -> bool {
        if let Some(index) = self.panes.iter().position(|p| p.id().0 == id.as_str()) {
            if index != self.active_index {
                self.panes[self.active_index].on_exit(state);
                self.active_index = index;
                self.panes[self.active_index].on_enter(state);
            }
            self.stack.clear();
            true
        } else {
            false
        }
    }

    /// Push current pane onto the stack and switch to a new pane (for modals/overlays)
    pub fn push_to(&mut self, id: PaneId, state: &AppState) -> bool {
        if let Some(index) = self.panes.iter().position(|p| p.id().0 == id.as_str()) {
            self.stack.push(self.active_index);
            self.panes[self.active_index].on_exit(state);
            self.active_index = index;
            self.panes[self.active_index].on_enter(state);
            true
        } else {
            false
        }
    }

    /// Pop the stack and return to the previous pane
    pub fn pop(&mut self, state: &AppState) -> bool {
        if let Some(prev_index) = self.stack.pop() {
            self.panes[self.active_index].on_exit(state);
            self.active_index = prev_index;
            self.panes[self.active_index].on_enter(state);
            true
        } else {
            false
        }
    }

    /// Process navigation actions from a routed action
    pub fn process_nav(&mut self, action: &crate::action::RoutedAction, state: &AppState) {
        use crate::action::UiAction;
        match action {
            crate::action::RoutedAction::Ui(UiAction::Nav(NavAction::SwitchPane(id))) => {
                self.switch_to(*id, state);
            }
            crate::action::RoutedAction::Ui(UiAction::Nav(NavAction::PushPane(id))) => {
                self.push_to(*id, state);
            }
            crate::action::RoutedAction::Ui(UiAction::Nav(NavAction::PopPane)) => {
                self.pop(state);
            }
            _ => {}
        }
    }

    /// Process navigation intents returned from dispatch
    pub fn process_nav_intents(&mut self, intents: &[NavIntent], state: &AppState) {
        for intent in intents {
            match intent {
                NavIntent::SwitchTo(id) => {
                    self.switch_to(*id, state);
                }
                NavIntent::PushTo(id) => {
                    self.push_to(*id, state);
                }
                NavIntent::Pop => {
                    self.pop(state);
                }
                NavIntent::ConditionalPop(pane_id) => {
                    if self.active().id().0 == pane_id.as_str() {
                        self.pop(state);
                    }
                }
                NavIntent::PopOrSwitchTo(fallback) => {
                    if !self.pop(state) {
                        self.switch_to(*fallback, state);
                    }
                }
                NavIntent::OpenFileBrowser(_)
                | NavIntent::OpenVstParams(_, _)
                | NavIntent::OpenTagPicker(_) => {
                    // Handled by main.rs which configures the pane before pushing
                }
            }
        }
    }

    /// Render the active pane to the buffer, with hint bar if applicable.
    pub fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let hints = self.panes[self.active_index].keymap().hint_bindings();
        if !hints.is_empty() && area.height > 2 {
            // Render pane with full area; draw_block auto-registers the hint anchor.
            buf.clear_hint_anchor();
            self.panes[self.active_index].render(area, buf, state);
            if let Some(anchor) = buf.hint_anchor() {
                // Anchored: hint hangs off the bottom of the dialog
                let hint_area = Rect {
                    x: anchor.x,
                    y: anchor.y + anchor.height,
                    width: anchor.width,
                    height: 1,
                };
                Self::render_hint_line(&hints, hint_area, buf);
            } else {
                // No dialog drawn — fall back to bottom of area
                Self::render_hint_line(&hints, area, buf);
            }
        } else {
            self.panes[self.active_index].render(area, buf, state);
        }
    }

    /// Render the hint bar at the bottom row of the given area.
    fn render_hint_line(hints: &[(String, &str)], area: Rect, buf: &mut RenderBuf) {
        let y = area.y + area.height - 1;
        let max_x = area.x + area.width;

        // Clear the hint row (it overlaps pane content area)
        let clear_style = Style::new();
        for cx in area.x..max_x {
            buf.set_cell(cx, y, ' ', clear_style);
        }

        let mut x = area.x + 1;

        let key_style = Style::new().fg(Color::CYAN);
        let label_style = Style::new().fg(Color::DARK_GRAY);

        for (i, (key, label)) in hints.iter().enumerate() {
            if i > 0 {
                x += 2; // double-space separator
            }
            let key_len = key.chars().count() as u16;
            let label_len = label.chars().count() as u16;
            let total = key_len + 1 + label_len; // key + space + label
            if x + total > max_x {
                break;
            }
            buf.draw_str(x, y, key, key_style);
            x += key_len + 1;
            buf.draw_str(x, y, label, label_style);
            x += label_len;
        }
    }

    /// Get the keymap of the active pane
    #[allow(dead_code)]
    pub fn active_keymap(&self) -> &Keymap {
        self.active().keymap()
    }

    /// Get all registered pane IDs
    #[allow(dead_code)]
    pub fn pane_ids(&self) -> Vec<&'static str> {
        self.panes.iter().map(|p| p.id().0).collect()
    }

    /// Get a mutable reference to a pane by ID, downcasted to a specific type
    pub fn get_pane_mut<T: 'static>(&mut self, id: &str) -> Option<&mut T> {
        self.panes
            .iter_mut()
            .find(|p| p.id().0 == id)
            .and_then(|p| p.as_any_mut().downcast_mut::<T>())
    }
}
