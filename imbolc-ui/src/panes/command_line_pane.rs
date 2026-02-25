use std::any::Any;

use imbolc_types::DomainAction;

use crate::repl;
use crate::state::AppState;
use crate::ui::layout_helpers::center_rect;
use crate::ui::widgets::TextInput;
use crate::ui::{
    Action, InputEvent, KeyCode, Keymap, NavAction, Palette, Pane, PaneIdStr, Rect, RenderBuf,
    Style,
};

pub struct CommandLinePane {
    keymap: Keymap,
    text_input: TextInput,
    output: Vec<String>,
    scroll: usize,
    pending_action: Option<DomainAction>,
    error: Option<String>,
    history: Vec<String>,
    history_index: Option<usize>,
    saved_input: String,
    kill_buffer: String,
}

impl CommandLinePane {
    pub fn new(keymap: Keymap) -> Self {
        let mut text_input = TextInput::new("");
        text_input.set_focused(true);
        Self {
            keymap,
            text_input,
            output: Vec::new(),
            scroll: 0,
            pending_action: None,
            error: None,
            history: Vec::new(),
            history_index: None,
            saved_input: String::new(),
            kill_buffer: String::new(),
        }
    }

    /// Called by runtime after pop to get the pending domain action.
    pub fn take_action(&mut self) -> Option<DomainAction> {
        self.pending_action.take()
    }

    fn history_prev(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let new_index = match self.history_index {
            None => {
                // First Up press — save current input, go to most recent
                self.saved_input = self.text_input.value().to_string();
                self.history.len() - 1
            }
            Some(0) => return, // already at oldest
            Some(i) => i - 1,
        };
        self.history_index = Some(new_index);
        self.text_input.set_value(&self.history[new_index]);
    }

    fn history_next(&mut self) {
        let i = match self.history_index {
            None => return, // not browsing
            Some(i) => i,
        };
        if i + 1 < self.history.len() {
            self.history_index = Some(i + 1);
            self.text_input.set_value(&self.history[i + 1]);
        } else {
            // Past the end — restore saved input
            self.history_index = None;
            let saved = self.saved_input.clone();
            self.text_input.set_value(&saved);
        }
    }
}

impl Pane for CommandLinePane {
    fn id(&self) -> PaneIdStr {
        PaneIdStr("command_line")
    }

    fn on_enter(&mut self, _state: &AppState) {
        self.text_input.set_value("");
        self.text_input.set_focused(true);
        self.error = None;
        self.history_index = None;
        self.saved_input.clear();
        // Keep output history across opens for scrollback
    }

    fn handle_action(
        &mut self,
        _action: crate::ui::action_id::ActionId,
        _event: &InputEvent,
        _state: &AppState,
    ) -> Action {
        Action::None
    }

    fn handle_raw_input(&mut self, event: &InputEvent, state: &AppState) -> Action {
        // Intercept ctrl-key readline bindings before fallthrough
        if event.modifiers.ctrl {
            if let KeyCode::Char(ch) = event.key {
                match ch {
                    'a' => {
                        self.text_input.move_to_line_start();
                        return Action::None;
                    }
                    'e' => {
                        self.text_input.move_to_line_end();
                        return Action::None;
                    }
                    'k' => {
                        // Kill cursor→end
                        let cursor = self.text_input.cursor();
                        let len = self.text_input.len();
                        if cursor < len {
                            self.kill_buffer = self.text_input.str_slice(cursor..len);
                            self.text_input.delete_range(cursor..len);
                        }
                        return Action::None;
                    }
                    'u' => {
                        // Kill start→cursor
                        let cursor = self.text_input.cursor();
                        if cursor > 0 {
                            self.kill_buffer = self.text_input.str_slice(0..cursor);
                            self.text_input.delete_range(0..cursor);
                        }
                        return Action::None;
                    }
                    'w' => {
                        // Kill word backward
                        let cursor = self.text_input.cursor();
                        if cursor > 0 {
                            let word_start = self.text_input.prev_word_start(cursor);
                            self.kill_buffer = self.text_input.str_slice(word_start..cursor);
                            self.text_input.delete_range(word_start..cursor);
                        }
                        return Action::None;
                    }
                    'y' => {
                        // Yank from kill buffer
                        if !self.kill_buffer.is_empty() {
                            let kb = self.kill_buffer.clone();
                            self.text_input.insert_str(&kb);
                        }
                        return Action::None;
                    }
                    'p' => {
                        self.history_prev();
                        return Action::None;
                    }
                    'n' => {
                        self.history_next();
                        return Action::None;
                    }
                    _ => {}
                }
            }
        }

        match event.key {
            KeyCode::Enter => {
                let input = self.text_input.value().trim().to_string();
                if input.is_empty() {
                    return Action::None;
                }

                // Push to history (skip consecutive duplicates)
                if self.history.last().is_none_or(|last| *last != input) {
                    self.history.push(input.clone());
                }
                self.history_index = None;
                self.saved_input.clear();

                match repl::parse_command(&input, state) {
                    Ok(repl::CommandResult::Action(domain)) => {
                        self.output.push(format!(": {}", input));
                        self.pending_action = Some(domain);
                        self.text_input.set_value("");
                        self.error = None;
                        Action::Nav(NavAction::PopPane)
                    }
                    Ok(repl::CommandResult::Output(text)) => {
                        self.output.push(format!(": {}", input));
                        for line in text.lines() {
                            self.output.push(line.to_string());
                        }
                        self.text_input.set_value("");
                        self.error = None;
                        // Scroll to bottom
                        self.scroll = self.output.len().saturating_sub(1);
                        Action::None
                    }
                    Ok(repl::CommandResult::Quit) => Action::Quit,
                    Err(msg) => {
                        self.error = Some(msg);
                        self.text_input.set_value("");
                        Action::None
                    }
                }
            }
            KeyCode::Escape => {
                self.error = None;
                Action::Nav(NavAction::PopPane)
            }
            KeyCode::Tab => {
                let current = self.text_input.value().to_string();
                let completions = repl::completion::complete_input(&current);
                if completions.len() == 1 {
                    let completed = format!("{} ", completions[0]);
                    self.text_input.set_value(&completed);
                } else if completions.len() > 1 {
                    // Find longest common prefix
                    let prefix = longest_common_prefix(&completions);
                    if prefix.len() > current.len() {
                        self.text_input.set_value(&prefix);
                    } else {
                        // Show available completions in output
                        self.output.push(format!(": {}", current));
                        self.output.push(completions.join("  "));
                        self.scroll = self.output.len().saturating_sub(1);
                    }
                }
                Action::None
            }
            KeyCode::Up => {
                self.history_prev();
                Action::None
            }
            KeyCode::Down => {
                self.history_next();
                Action::None
            }
            KeyCode::PageUp => {
                self.scroll = self.scroll.saturating_sub(1);
                Action::None
            }
            KeyCode::PageDown => {
                if !self.output.is_empty() {
                    self.scroll = (self.scroll + 1).min(self.output.len().saturating_sub(1));
                }
                Action::None
            }
            _ => {
                self.text_input.handle_input(event);
                self.error = None;
                // Any manual edit resets history browsing
                self.history_index = None;
                Action::None
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, state: &AppState) {
        let p = Palette::from(&state.session.theme);
        let rect = center_rect(area, 97, 29);

        // Clear background
        let bg_style = Style::new().bg(p.bg);
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                buf.set_cell(x, y, ' ', bg_style);
            }
        }

        let border_style = Style::new().fg(p.accent);
        let inner = buf.draw_block(rect, " Command Line ", border_style, border_style);

        if inner.height == 0 || inner.width == 0 {
            return;
        }

        // Output area (top part of inner)
        let output_area_height = inner.height.saturating_sub(2) as usize; // reserve 1 for divider, 1 for input
        if !self.output.is_empty() && output_area_height > 0 {
            let total_lines = self.output.len();
            // Ensure scroll doesn't exceed bounds
            let max_scroll = total_lines.saturating_sub(output_area_height);
            if self.scroll > max_scroll {
                self.scroll = max_scroll;
            }

            let start = self.scroll;
            let end = (start + output_area_height).min(total_lines);
            for (i, line_idx) in (start..end).enumerate() {
                let y = inner.y + i as u16;
                let line = &self.output[line_idx];
                let style = if line.starts_with(": ") {
                    Style::new().fg(p.accent)
                } else {
                    Style::new().fg(p.dim)
                };
                let display = if line.len() > inner.width as usize {
                    &line[..inner.width as usize]
                } else {
                    line.as_str()
                };
                buf.draw_line(Rect::new(inner.x, y, inner.width, 1), &[(display, style)]);
            }
        }

        // Error or divider line
        let divider_y = inner.y + inner.height.saturating_sub(2);
        if let Some(ref error) = self.error {
            let err_display = if error.len() > inner.width as usize {
                &error[..inner.width as usize]
            } else {
                error.as_str()
            };
            buf.draw_line(
                Rect::new(inner.x, divider_y, inner.width, 1),
                &[(err_display, Style::new().fg(p.mute_color))],
            );
        } else {
            let divider = "\u{2500}".repeat(inner.width as usize);
            buf.draw_line(
                Rect::new(inner.x, divider_y, inner.width, 1),
                &[(&divider, Style::new().fg(p.muted))],
            );
        }

        // Input line (bottom)
        let input_y = inner.y + inner.height.saturating_sub(1);
        buf.draw_line(
            Rect::new(inner.x, input_y, 2, 1),
            &[(": ", Style::new().fg(p.accent).bold())],
        );
        self.text_input.render_buf(
            buf.raw_buf(),
            inner.x + 2,
            input_y,
            inner.width.saturating_sub(2),
            &p,
        );
        if let Some((cx, cy)) = self.text_input.screen_cursor() {
            buf.set_cursor_position(cx, cy);
        }
    }

    fn keymap(&self) -> &Keymap {
        &self.keymap
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let first = &strings[0];
    let mut len = first.len();
    for s in &strings[1..] {
        len = len.min(s.len());
        for (i, (a, b)) in first.chars().zip(s.chars()).enumerate() {
            if a != b {
                len = len.min(i);
                break;
            }
        }
    }
    first[..len].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::input::Modifiers;

    fn make_pane() -> CommandLinePane {
        CommandLinePane::new(Keymap::new())
    }

    fn char_event(ch: char) -> InputEvent {
        InputEvent::new(KeyCode::Char(ch), Modifiers::none())
    }

    fn ctrl_event(ch: char) -> InputEvent {
        InputEvent::new(KeyCode::Char(ch), Modifiers::ctrl())
    }

    fn key_event(key: KeyCode) -> InputEvent {
        InputEvent::new(key, Modifiers::none())
    }

    /// Helper: type a string into the pane's text input
    fn type_str(pane: &mut CommandLinePane, s: &str) {
        for ch in s.chars() {
            pane.text_input.handle_input(&char_event(ch));
        }
    }

    /// Helper: populate history by setting input + simulating Enter logic
    fn push_history(pane: &mut CommandLinePane, cmd: &str) {
        if pane.history.last().is_none_or(|last| *last != cmd) {
            pane.history.push(cmd.to_string());
        }
        pane.history_index = None;
        pane.saved_input.clear();
    }

    // -- History tests --

    #[test]
    fn history_up_down_cycling() {
        let mut pane = make_pane();
        push_history(&mut pane, "first");
        push_history(&mut pane, "second");
        push_history(&mut pane, "third");

        // Up → most recent
        pane.history_prev();
        assert_eq!(pane.text_input.value(), "third");

        pane.history_prev();
        assert_eq!(pane.text_input.value(), "second");

        pane.history_prev();
        assert_eq!(pane.text_input.value(), "first");

        // Up at oldest is no-op
        pane.history_prev();
        assert_eq!(pane.text_input.value(), "first");

        // Down walks forward
        pane.history_next();
        assert_eq!(pane.text_input.value(), "second");

        pane.history_next();
        assert_eq!(pane.text_input.value(), "third");

        // Down past end restores saved input
        pane.history_next();
        assert_eq!(pane.text_input.value(), "");
    }

    #[test]
    fn history_preserves_unsaved_input() {
        let mut pane = make_pane();
        push_history(&mut pane, "old command");

        // Type partial input
        type_str(&mut pane, "partial");

        // Up saves it and shows history
        pane.history_prev();
        assert_eq!(pane.text_input.value(), "old command");

        // Down restores the partial input
        pane.history_next();
        assert_eq!(pane.text_input.value(), "partial");
    }

    #[test]
    fn history_skips_consecutive_duplicates() {
        let mut pane = make_pane();
        push_history(&mut pane, "same");
        push_history(&mut pane, "same");
        push_history(&mut pane, "same");
        assert_eq!(pane.history.len(), 1);
    }

    #[test]
    fn on_enter_resets_history_index() {
        let mut pane = make_pane();
        push_history(&mut pane, "command");
        pane.history_prev();
        assert!(pane.history_index.is_some());

        // Simulate on_enter
        let state = AppState::default();
        pane.on_enter(&state);
        assert!(pane.history_index.is_none());
        assert!(pane.saved_input.is_empty());
    }

    // -- Readline cursor movement tests --

    #[test]
    fn ctrl_a_moves_to_start() {
        let mut pane = make_pane();
        type_str(&mut pane, "hello");
        assert!(pane.text_input.cursor() > 0);

        pane.handle_raw_input(&ctrl_event('a'), &AppState::default());
        assert_eq!(pane.text_input.cursor(), 0);
    }

    #[test]
    fn ctrl_e_moves_to_end() {
        let mut pane = make_pane();
        type_str(&mut pane, "hello");
        pane.text_input.move_to_line_start();
        assert_eq!(pane.text_input.cursor(), 0);

        pane.handle_raw_input(&ctrl_event('e'), &AppState::default());
        assert_eq!(pane.text_input.cursor(), 5);
    }

    // -- Kill/yank tests --

    #[test]
    fn ctrl_k_kills_to_end() {
        let mut pane = make_pane();
        type_str(&mut pane, "hello world");
        pane.text_input.set_cursor(5);

        pane.handle_raw_input(&ctrl_event('k'), &AppState::default());
        assert_eq!(pane.text_input.value(), "hello");
        assert_eq!(pane.kill_buffer, " world");
    }

    #[test]
    fn ctrl_u_kills_to_start() {
        let mut pane = make_pane();
        type_str(&mut pane, "hello world");
        pane.text_input.set_cursor(5);

        pane.handle_raw_input(&ctrl_event('u'), &AppState::default());
        assert_eq!(pane.text_input.value(), " world");
        assert_eq!(pane.kill_buffer, "hello");
    }

    #[test]
    fn ctrl_y_yanks_killed_text() {
        let mut pane = make_pane();
        type_str(&mut pane, "hello world");
        pane.text_input.set_cursor(5);

        // Kill to end
        pane.handle_raw_input(&ctrl_event('k'), &AppState::default());
        assert_eq!(pane.text_input.value(), "hello");

        // Yank it back
        pane.handle_raw_input(&ctrl_event('y'), &AppState::default());
        assert_eq!(pane.text_input.value(), "hello world");
    }

    #[test]
    fn ctrl_w_kills_word_backward() {
        let mut pane = make_pane();
        type_str(&mut pane, "hello world");
        // Cursor is at the end (pos 11)

        pane.handle_raw_input(&ctrl_event('w'), &AppState::default());
        // Should kill "world" (the last word)
        assert_eq!(pane.text_input.value(), "hello ");
    }

    #[test]
    fn ctrl_k_ctrl_u_at_boundary_no_op() {
        let mut pane = make_pane();
        type_str(&mut pane, "test");

        // Ctrl+K at end — nothing to kill
        pane.handle_raw_input(&ctrl_event('k'), &AppState::default());
        assert_eq!(pane.text_input.value(), "test");

        // Ctrl+U at start — nothing to kill
        pane.text_input.move_to_line_start();
        pane.handle_raw_input(&ctrl_event('u'), &AppState::default());
        assert_eq!(pane.text_input.value(), "test");
    }

    // -- Ctrl+P / Ctrl+N history --

    #[test]
    fn ctrl_p_n_navigate_history() {
        let mut pane = make_pane();
        push_history(&mut pane, "alpha");
        push_history(&mut pane, "beta");

        pane.handle_raw_input(&ctrl_event('p'), &AppState::default());
        assert_eq!(pane.text_input.value(), "beta");

        pane.handle_raw_input(&ctrl_event('p'), &AppState::default());
        assert_eq!(pane.text_input.value(), "alpha");

        pane.handle_raw_input(&ctrl_event('n'), &AppState::default());
        assert_eq!(pane.text_input.value(), "beta");

        pane.handle_raw_input(&ctrl_event('n'), &AppState::default());
        assert_eq!(pane.text_input.value(), "");
    }

    // -- PageUp/PageDown scroll --

    #[test]
    fn page_up_down_scroll_output() {
        let mut pane = make_pane();
        pane.output = vec!["line1".into(), "line2".into(), "line3".into()];
        pane.scroll = 2;

        pane.handle_raw_input(&key_event(KeyCode::PageUp), &AppState::default());
        assert_eq!(pane.scroll, 1);

        pane.handle_raw_input(&key_event(KeyCode::PageDown), &AppState::default());
        assert_eq!(pane.scroll, 2);
    }

    // -- Up/Down now do history, not scroll --

    #[test]
    fn up_down_do_history_not_scroll() {
        let mut pane = make_pane();
        pane.output = vec!["line1".into(), "line2".into()];
        pane.scroll = 1;
        push_history(&mut pane, "cmd");

        pane.handle_raw_input(&key_event(KeyCode::Up), &AppState::default());
        // Should show history, not change scroll
        assert_eq!(pane.text_input.value(), "cmd");
    }
}
