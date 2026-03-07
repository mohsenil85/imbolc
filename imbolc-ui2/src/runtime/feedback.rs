use std::time::Instant;

use super::AppRuntime;
use crate::action::{self, AudioEffect, IoFeedback};
use crate::ui::status_bar::StatusLevel;
use imbolc_types::AudioFeedback;

impl AppRuntime {
    /// Drain I/O feedback (save/load completions).
    pub(crate) fn drain_io_feedback(&mut self) {
        while let Ok(feedback) = self.io_rx.try_recv() {
            match feedback {
                IoFeedback::SaveComplete { id, path, result } => {
                    if id != self.dispatcher.state().io.generation.save {
                        continue;
                    }
                    match result {
                        Ok(name) => {
                            let state = self.dispatcher.state_mut();
                            state.project.path = Some(path.clone());
                            state.project.dirty = false;
                            self.chrome.set_project_name(name);
                            self.chrome
                                .status_bar
                                .push("Project saved", StatusLevel::Info);
                            self.autosave_id = self.autosave_id.wrapping_add(1);
                            self.autosave_in_progress = false;
                            let _ = std::fs::remove_file(&self.autosave_path);
                            self.last_autosave_at = Instant::now();
                        }
                        Err(e) => {
                            let msg = format!("Save failed: {}", e);
                            self.chrome.status_bar.push(&msg, StatusLevel::Error);
                        }
                    }
                }
                IoFeedback::AutosaveComplete { id, path, result } => {
                    if id != self.autosave_id {
                        let _ = std::fs::remove_file(path);
                        continue;
                    }
                    self.autosave_in_progress = false;
                    if let Err(e) = result {
                        let msg = format!("Autosave failed: {}", e);
                        self.chrome.status_bar.push(&msg, StatusLevel::Error);
                    }
                }
                IoFeedback::LoadComplete { id, path, result } => {
                    if id != self.dispatcher.state().io.generation.load {
                        continue;
                    }
                    match result {
                        Ok((new_session, new_instruments, name)) => {
                            let recovering = path == self.autosave_path;
                            {
                                let state = self.dispatcher.state_mut();
                                let global_theme = state.session.theme.clone();
                                state.undo_history.clear();
                                state.session = new_session;
                                state.session.theme = global_theme;
                                state.tracks = new_instruments;
                                state.tracks.rebuild_index();
                                if recovering {
                                    state.project.path = None;
                                    state.project.dirty = true;
                                } else {
                                    state.project.path = Some(path);
                                    state.project.dirty = false;
                                }
                            }
                            if recovering {
                                self.chrome
                                    .set_project_name("autosave-recovered".to_string());
                            } else {
                                self.chrome.set_project_name(name);
                            }
                            self.last_autosave_at = Instant::now();
                            self.autosave_id = self.autosave_id.wrapping_add(1);
                            self.autosave_in_progress = false;
                            self.pending_audio_effects.extend(AudioEffect::all());
                            self.needs_full_sync = true;
                            self.chrome
                                .status_bar
                                .push("Project loaded", StatusLevel::Info);
                        }
                        Err(e) => {
                            let msg = format!("Load failed: {}", e);
                            self.chrome.status_bar.push(&msg, StatusLevel::Error);
                        }
                    }
                }
                IoFeedback::ImportSynthDefComplete { .. }
                | IoFeedback::ImportSynthDefLoaded { .. } => {
                    // Not yet supported in ui2
                }
            }
        }
    }

    /// Drain audio feedback (playhead, meters, status).
    pub(crate) fn drain_audio_feedback(&mut self) {
        let mut had_playhead = false;
        for feedback in self.audio.drain_feedback() {
            if matches!(feedback, AudioFeedback::PlayheadPosition(_)) {
                had_playhead = true;
                continue;
            }
            self.render_needed = true;
            let mut r = self.dispatcher.dispatch_domain(
                &action::DomainAction::AudioFeedback(feedback),
                &mut self.audio,
            );
            if r.needs_full_sync {
                self.needs_full_sync = true;
            }
            self.pending_audio_effects
                .extend(std::mem::take(&mut r.audio_effects));
        }
        if had_playhead {
            self.render_needed = true;
        }
    }

    /// Poll MIDI events (basic passthrough for now).
    pub(crate) fn drain_midi_events(&mut self) {
        // MIDI dispatch requires the midi_dispatch module from imbolc-ui.
        // For MVP, we just drain events to prevent queue buildup.
        let _ = self.midi_input.poll_events();
    }
}
