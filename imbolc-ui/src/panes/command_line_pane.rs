use std::any::Any;

use imbolc_types::DomainAction;

use crate::repl;
use crate::state::AppState;
use crate::ui::layout_helpers::center_rect;
use crate::ui::widgets::TextInput;
use crate::ui::{
    Action, Color, InputEvent, KeyCode, Keymap, NavAction, Pane, Rect, RenderBuf, Style,
};

pub struct CommandLinePane {
    keymap: Keymap,
    text_input: TextInput,
    output: Vec<String>,
    scroll: usize,
    pending_action: Option<DomainAction>,
    error: Option<String>,
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
        }
    }

    /// Called by runtime after pop to get the pending domain action.
    pub fn take_action(&mut self) -> Option<DomainAction> {
        self.pending_action.take()
    }
}

impl Pane for CommandLinePane {
    fn id(&self) -> &'static str {
        "command_line"
    }

    fn on_enter(&mut self, _state: &AppState) {
        self.text_input.set_value("");
        self.text_input.set_focused(true);
        self.error = None;
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
        match event.key {
            KeyCode::Enter => {
                let input = self.text_input.value().trim().to_string();
                if input.is_empty() {
                    return Action::None;
                }

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
                self.scroll = self.scroll.saturating_sub(1);
                Action::None
            }
            KeyCode::Down => {
                if !self.output.is_empty() {
                    self.scroll = (self.scroll + 1).min(self.output.len().saturating_sub(1));
                }
                Action::None
            }
            _ => {
                self.text_input.handle_input(event);
                self.error = None;
                Action::None
            }
        }
    }

    fn render(&mut self, area: Rect, buf: &mut RenderBuf, _state: &AppState) {
        let width = 70u16.min(area.width.saturating_sub(4));
        let output_rows = 10u16.min(area.height.saturating_sub(6));
        let total_height = output_rows + 4; // border(1) + output + error/divider(1) + input(1) + border(1)
        let rect = center_rect(area, width, total_height);

        // Clear background
        let bg_style = Style::new().bg(Color::new(20, 20, 30));
        for y in rect.y..rect.y + rect.height {
            for x in rect.x..rect.x + rect.width {
                buf.set_cell(x, y, ' ', bg_style);
            }
        }

        let border_style = Style::new().fg(Color::CYAN);
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
                    Style::new().fg(Color::CYAN)
                } else {
                    Style::new().fg(Color::GRAY)
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
                &[(err_display, Style::new().fg(Color::MUTE_COLOR))],
            );
        } else {
            let divider = "\u{2500}".repeat(inner.width as usize);
            buf.draw_line(
                Rect::new(inner.x, divider_y, inner.width, 1),
                &[(&divider, Style::new().fg(Color::DARK_GRAY))],
            );
        }

        // Input line (bottom)
        let input_y = inner.y + inner.height.saturating_sub(1);
        buf.draw_line(
            Rect::new(inner.x, input_y, 2, 1),
            &[(": ", Style::new().fg(Color::CYAN).bold())],
        );
        self.text_input.render_buf(
            buf.raw_buf(),
            inner.x + 2,
            input_y,
            inner.width.saturating_sub(2),
        );
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
