use crate::buffer::BufferRegistry;
use crate::state::AppState;
use crate::ui::action_id::{ActionId, GlobalActionId};
use crate::ui::input::{AppEvent, InputEvent, KeyCode, MouseEvent};
use crate::ui::layer::{LayerResult, LayerStack};
use crate::window::WindowTree;
use imbolc_types::action::Action;

/// Actions that affect the window system itself.
#[allow(dead_code)]
pub enum WindowAction {
    SplitHorizontal,
    SplitVertical,
    CloseFocused,
    CloseOthers,
    FocusNext,
    FocusPrev,
}

/// Result of processing input through the chord state machine.
pub enum InputResult {
    /// A domain/ui action from the buffer or global handler.
    BufferAction(Action),
    /// A window management action.
    Window(WindowAction),
    /// Quit the application.
    Quit,
    /// Nothing happened.
    None,
    /// Resize event.
    Resize,
}

/// Chord state for Ctrl+x prefix commands.
enum ChordState {
    Normal,
    CtrlXPending,
}

/// Input processor with chord state machine.
pub struct InputProcessor {
    chord_state: ChordState,
}

impl InputProcessor {
    pub fn new() -> Self {
        Self {
            chord_state: ChordState::Normal,
        }
    }

    /// Process an app event, returning the resulting action.
    pub fn process(
        &mut self,
        event: AppEvent,
        layer_stack: &LayerStack,
        tree: &WindowTree,
        registry: &mut BufferRegistry,
        state: &AppState,
        last_area: ratatui::layout::Rect,
    ) -> InputResult {
        match event {
            AppEvent::Resize(_, _) => InputResult::Resize,
            AppEvent::Mouse(mouse_event) => {
                self.process_mouse(mouse_event, tree, registry, state, last_area)
            }
            AppEvent::Key(key_event) => {
                self.process_key(key_event, layer_stack, tree, registry, state)
            }
        }
    }

    fn process_key(
        &mut self,
        event: InputEvent,
        layer_stack: &LayerStack,
        tree: &WindowTree,
        registry: &mut BufferRegistry,
        state: &AppState,
    ) -> InputResult {
        // Check chord state
        match self.chord_state {
            ChordState::CtrlXPending => {
                self.chord_state = ChordState::Normal;
                if let Some(action) = self.resolve_ctrl_x_chord(&event) {
                    return action;
                }
                // Fall through — not a recognized chord, treat as normal input
            }
            ChordState::Normal => {
                // Check for Ctrl+x to start chord
                if event.modifiers.ctrl && matches!(event.key, KeyCode::Char('x')) {
                    self.chord_state = ChordState::CtrlXPending;
                    return InputResult::None;
                }
            }
        }

        // Layer resolution
        match layer_stack.resolve(&event) {
            LayerResult::Action(action_id) => {
                // Check for global actions
                if let ActionId::Global(global) = action_id {
                    match global {
                        GlobalActionId::Quit => return InputResult::Quit,
                        _ => {}
                    }
                }

                // Forward to focused buffer
                let buf_id = tree.focused_buffer();
                if let Some(buffer) = registry.get_mut(&buf_id) {
                    let action = buffer.handle_action(action_id, &event, state);
                    InputResult::BufferAction(action)
                } else {
                    InputResult::None
                }
            }
            LayerResult::Blocked | LayerResult::Unresolved => {
                // Forward raw input to focused buffer
                let buf_id = tree.focused_buffer();
                if let Some(buffer) = registry.get_mut(&buf_id) {
                    let action = buffer.handle_raw_input(&event, state);
                    InputResult::BufferAction(action)
                } else {
                    InputResult::None
                }
            }
        }
    }

    /// Resolve the second key of a Ctrl+x chord.
    fn resolve_ctrl_x_chord(&self, event: &InputEvent) -> Option<InputResult> {
        match event.key {
            // Ctrl+x 2 → split vertical (top/bottom)
            KeyCode::Char('2') if !event.modifiers.ctrl => {
                Some(InputResult::Window(WindowAction::SplitVertical))
            }
            // Ctrl+x 3 → split horizontal (left/right)
            KeyCode::Char('3') if !event.modifiers.ctrl => {
                Some(InputResult::Window(WindowAction::SplitHorizontal))
            }
            // Ctrl+x 0 → close focused window
            KeyCode::Char('0') if !event.modifiers.ctrl => {
                Some(InputResult::Window(WindowAction::CloseFocused))
            }
            // Ctrl+x 1 → close all other windows
            KeyCode::Char('1') if !event.modifiers.ctrl => {
                Some(InputResult::Window(WindowAction::CloseOthers))
            }
            // Ctrl+x o → focus next window
            KeyCode::Char('o') if !event.modifiers.ctrl => {
                Some(InputResult::Window(WindowAction::FocusNext))
            }
            _ => None,
        }
    }

    fn process_mouse(
        &mut self,
        mouse_event: MouseEvent,
        tree: &WindowTree,
        registry: &mut BufferRegistry,
        state: &AppState,
        last_area: ratatui::layout::Rect,
    ) -> InputResult {
        // Hit-test which window the click landed in
        let window_area = crate::runtime::window_area(last_area);
        if let Some(window_id) =
            crate::window::render::hit_test(tree, window_area, mouse_event.column, mouse_event.row)
        {
            // Find the buffer in the clicked window and forward the mouse event
            if let Some(buf_id) = tree.find_buffer_for(window_id) {
                if let Some(buffer) = registry.get_mut(&buf_id) {
                    let action = buffer.handle_mouse(&mouse_event, last_area, state);
                    return InputResult::BufferAction(action);
                }
            }
        }
        InputResult::None
    }
}
