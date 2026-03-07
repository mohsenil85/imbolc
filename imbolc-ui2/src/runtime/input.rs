use std::time::Duration;

use super::AppRuntime;
use crate::input::{InputResult, WindowAction};
use crate::ui::input::InputSource;

impl AppRuntime {
    /// Process input events. Returns true if the app should quit.
    pub(crate) fn process_events(
        &mut self,
        backend: &mut crate::ui::RatatuiBackend,
        poll_timeout: Duration,
    ) -> std::io::Result<bool> {
        let mut events_processed = 0u8;

        loop {
            let timeout = if events_processed == 0 {
                poll_timeout
            } else {
                Duration::ZERO
            };
            let app_event = match backend.poll_event(timeout) {
                Some(e) => e,
                None => break,
            };
            events_processed += 1;

            let result = self.input_processor.process(
                app_event,
                &self.layer_stack,
                &self.tree,
                &mut self.registry,
                self.dispatcher.state(),
                self.last_area,
            );

            match result {
                InputResult::Quit => return Ok(true),
                InputResult::Resize => {
                    self.render_needed = true;
                }
                InputResult::Window(action) => {
                    self.handle_window_action(action);
                }
                InputResult::BufferAction(action) => {
                    use imbolc_types::RoutedAction;
                    let routed = action.route();
                    match routed {
                        RoutedAction::Domain(ref domain) => {
                            let mut r = self.dispatcher.dispatch_domain(domain, &mut self.audio);
                            if r.needs_full_sync {
                                self.needs_full_sync = true;
                            }
                            self.pending_audio_effects
                                .extend(std::mem::take(&mut r.audio_effects));
                        }
                        RoutedAction::Ui(imbolc_types::UiAction::Quit) => {
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
                InputResult::None => {}
            }

            if events_processed >= 16 {
                break;
            }
        }

        if events_processed > 0 {
            self.render_needed = true;
        }

        Ok(false)
    }

    fn handle_window_action(&mut self, action: WindowAction) {
        match action {
            WindowAction::SplitHorizontal => {
                self.tree.split(crate::window::Direction::Horizontal);
            }
            WindowAction::SplitVertical => {
                self.tree.split(crate::window::Direction::Vertical);
            }
            WindowAction::CloseFocused => {
                self.tree.close_focused();
            }
            WindowAction::CloseOthers => {
                self.tree.close_others();
            }
            WindowAction::FocusNext => {
                self.tree.focus_next();
                self.sync_buffer_layer();
            }
            WindowAction::FocusPrev => {
                self.tree.focus_prev();
                self.sync_buffer_layer();
            }
        }
        self.render_needed = true;
    }

    fn sync_buffer_layer(&mut self) {
        let buf_id = self.tree.focused_buffer();
        if let Some(buffer) = self.registry.get(&buf_id) {
            self.layer_stack.set_buffer_layer(buffer.layer_name());
        }
    }

    /// Process time-based buffer updates.
    pub(crate) fn process_tick(&mut self) {
        let buf_id = self.tree.focused_buffer();
        if let Some(buffer) = self.registry.get_mut(&buf_id) {
            let tick_actions = buffer.tick(self.dispatcher.state());
            if !tick_actions.is_empty() {
                self.render_needed = true;
            }
            for action in tick_actions {
                use imbolc_types::RoutedAction;
                if let RoutedAction::Domain(ref domain) = action.route() {
                    let r = self.dispatcher.dispatch_domain(domain, &mut self.audio);
                    if r.needs_full_sync {
                        self.needs_full_sync = true;
                    }
                    self.pending_audio_effects.extend(r.audio_effects);
                }
            }
        }
    }
}
