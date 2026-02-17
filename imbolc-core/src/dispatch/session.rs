use crate::action::{AudioEffect, DispatchResult, IoFeedback, NavIntent, PaneId, SessionAction};
use crate::scd_parser;
use crate::state::{AppState, CustomSynthDef, ParamSpec};
use imbolc_audio::AudioHandle;
use imbolc_types::DomainAction;
use std::path::PathBuf;
use std::sync::mpsc::Sender;

use super::default_rack_path;
use super::server::compile_synthdef;

fn dispatch_save(
    path: PathBuf,
    state: &mut AppState,
    audio: &mut AudioHandle,
    io_tx: &Sender<IoFeedback>,
    result: &mut DispatchResult,
) {
    // Mark save as in progress
    state.io.save_in_progress = true;
    state.io.last_io_error = None;

    // piano_roll.time_signature/bpm are now kept in sync by SessionState setters
    let session = state.session.clone();
    let instruments = state.instruments.clone();
    let tx = io_tx.clone();
    let save_id = state.io.generation.next_save();

    std::thread::spawn(move || {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let res = crate::state::persistence::save_project(&path, &session, &instruments)
            .map(|_| {
                path.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("default")
                    .to_string()
            })
            .map_err(|e| e.to_string());

        let _ = tx.send(IoFeedback::SaveComplete {
            id: save_id,
            path,
            result: res,
        });
    });

    result.push_status(audio.status(), "Saving...");
}

fn dispatch_load(
    path: PathBuf,
    state: &mut AppState,
    audio: &mut AudioHandle,
    io_tx: &Sender<IoFeedback>,
    result: &mut DispatchResult,
) {
    // Mark load as in progress
    state.io.load_in_progress = true;
    state.io.last_io_error = None;

    let tx = io_tx.clone();
    let load_id = state.io.generation.next_load();

    std::thread::spawn(move || {
        let res = if path.exists() {
            crate::state::persistence::load_project(&path)
                .map(|(session, instruments)| {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("default")
                        .to_string();
                    (session, instruments, name)
                })
                .map_err(|e| e.to_string())
        } else {
            Err("Project file not found".to_string())
        };

        let _ = tx.send(IoFeedback::LoadComplete {
            id: load_id,
            path,
            result: res,
        });
    });

    result.push_status(audio.status(), "Loading...");
}

pub(super) fn dispatch_session(
    action: &SessionAction,
    state: &mut AppState,
    audio: &mut AudioHandle,
    io_tx: &Sender<IoFeedback>,
) -> DispatchResult {
    let mut result = DispatchResult::none();

    match action {
        SessionAction::Save => {
            let path = state.project.path.clone().unwrap_or_else(default_rack_path);
            dispatch_save(path, state, audio, io_tx, &mut result);
        }
        SessionAction::SaveAs(ref path) => {
            let path = path.clone();
            dispatch_save(path, state, audio, io_tx, &mut result);
            result.push_nav(NavIntent::ConditionalPop(PaneId::SaveAs));
        }
        SessionAction::Load => {
            let path = state.project.path.clone().unwrap_or_else(default_rack_path);
            dispatch_load(path, state, audio, io_tx, &mut result);
            result.push_nav(NavIntent::ConditionalPop(PaneId::Confirm));
            result.push_nav(NavIntent::ConditionalPop(PaneId::ProjectBrowser));
        }
        SessionAction::LoadFrom(ref path) => {
            let path = path.clone();
            dispatch_load(path, state, audio, io_tx, &mut result);
            result.push_nav(NavIntent::ConditionalPop(PaneId::Confirm));
            result.push_nav(NavIntent::ConditionalPop(PaneId::ProjectBrowser));
        }
        SessionAction::NewProject => {
            let defaults = state.project.default_settings.clone();
            state.session = crate::state::SessionState::new_with_defaults(
                defaults,
                crate::state::session::DEFAULT_BUS_COUNT,
            );
            state.instruments = crate::state::InstrumentState::new();
            state.project.path = None;
            state.project.dirty = false;
            state.undo_history.clear();
            result.audio_effects = AudioEffect::all();
            result.project_name = Some("untitled".to_string());
            result.push_nav(NavIntent::ConditionalPop(PaneId::Confirm));
            result.push_nav(NavIntent::ConditionalPop(PaneId::ProjectBrowser));
            result.push_nav(NavIntent::SwitchTo(PaneId::Add));
        }
        SessionAction::UpdateSession(_) => {
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.push_nav(NavIntent::PopOrSwitchTo(PaneId::TrackList));
            result.audio_effects.push(AudioEffect::RebuildSession);
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
        }
        SessionAction::UpdateSessionLive(_) => {
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.audio_effects.push(AudioEffect::RebuildSession);
            result.audio_effects.push(AudioEffect::UpdatePianoRoll);
        }
        SessionAction::OpenFileBrowser(ref file_action) => {
            // Reducer is a no-op for navigation; delegate for consistency
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.push_nav(NavIntent::OpenFileBrowser(file_action.clone()));
        }
        SessionAction::ImportCustomSynthDef(ref path) => {
            let path = path.clone();
            let tx = io_tx.clone();
            let import_id = state.io.generation.next_import_synthdef();

            std::thread::spawn(move || {
                // Read and parse the .scd file
                let res = match std::fs::read_to_string(&path) {
                    Ok(content) => {
                        match scd_parser::parse_scd_file(&content) {
                            Ok(parsed) => {
                                // Create params with inferred ranges
                                let params: Vec<ParamSpec> = parsed
                                    .params
                                    .iter()
                                    .map(|(name, default)| {
                                        let (min, max) =
                                            scd_parser::infer_param_range(name, *default);
                                        ParamSpec {
                                            name: name.clone(),
                                            default: *default,
                                            min,
                                            max,
                                        }
                                    })
                                    .collect();

                                // Create the custom synthdef entry
                                let synthdef_name = parsed.name.clone();
                                let custom = CustomSynthDef {
                                    id: imbolc_types::CustomSynthDefId::new(0), // Will be set by registry.add()
                                    name: parsed.name.clone(),
                                    synthdef_name: synthdef_name.clone(),
                                    source_path: path.clone(),
                                    params,
                                };

                                // Copy the .scd file to the config synthdefs directory
                                let config_dir = crate::paths::custom_synthdefs_dir();
                                let _ = std::fs::create_dir_all(&config_dir);

                                // Copy .scd file
                                if let Some(filename) = path.file_name() {
                                    let dest = config_dir.join(filename);
                                    let _ = std::fs::copy(&path, &dest);
                                }

                                // Compile the synthdef
                                match compile_synthdef(&path, &config_dir, &synthdef_name) {
                                    Ok(scsyndef_path) => Ok((custom, synthdef_name, scsyndef_path)),
                                    Err(e) => Err(format!("Failed to compile synthdef: {}", e)),
                                }
                            }
                            Err(e) => Err(format!("Failed to parse .scd file: {}", e)),
                        }
                    }
                    Err(e) => Err(format!("Failed to read .scd file: {}", e)),
                };

                let _ = tx.send(IoFeedback::ImportSynthDefComplete {
                    id: import_id,
                    result: res,
                });
            });

            result.push_status(audio.status(), "Importing SynthDef...");
            result.push_nav(NavIntent::Pop);
        }
        SessionAction::AdjustHumanizeVelocity(_) => {
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.audio_effects.push(AudioEffect::RebuildSession);
        }
        SessionAction::AdjustHumanizeTiming(_) => {
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.audio_effects.push(AudioEffect::RebuildSession);
        }
        SessionAction::ImportVstPlugin(ref path, kind) => {
            use crate::state::vst_plugin::{VstParamSpec, VstPlugin};

            let kind = *kind;

            // Extract display name from filename
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "VST Plugin".to_string());

            // Probe the VST3 binary for parameter metadata
            let params = if path.extension().and_then(|e| e.to_str()) == Some("vst3") {
                match crate::vst3_probe::probe_vst3_params(path) {
                    Ok(probed) => probed
                        .iter()
                        .map(|p| VstParamSpec {
                            index: p.index as u32,
                            name: p.name.clone(),
                            default: p.default_normalized as f32,
                            label: if p.units.is_empty() {
                                None
                            } else {
                                Some(p.units.clone())
                            },
                        })
                        .collect(),
                    Err(_) => vec![], // Probe failed — will discover via OSC later
                }
            } else {
                vec![]
            };

            let param_count = params.len();
            let plugin = VstPlugin {
                id: imbolc_types::VstPluginId::new(0), // Will be set by registry.add()
                name: name.clone(),
                plugin_path: path.clone(),
                kind,
                params,
            };

            let _id = state.session.vst_plugins.add(plugin);

            let status_msg = if param_count > 0 {
                format!("Imported VST: {} ({} params)", name, param_count)
            } else {
                format!("Imported VST: {}", name)
            };
            result.push_status(audio.status(), &status_msg);

            result.push_nav(NavIntent::Pop);
            result.audio_effects.push(AudioEffect::RebuildSession);
        }
        SessionAction::ToggleMasterMute => {
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.audio_effects.push(AudioEffect::RebuildSession);
            result.audio_effects.push(AudioEffect::UpdateMixerParams);
        }
        SessionAction::CycleTheme => {
            imbolc_types::reduce::reduce_action(
                &DomainAction::Session(action.clone()),
                &mut state.instruments,
                &mut state.session,
            );
            result.push_status(
                audio.status(),
                format!("Theme: {}", state.session.theme.name),
            );
        }
        SessionAction::CreateCheckpoint(ref label) => {
            let path = state.project.path.clone().unwrap_or_else(default_rack_path);
            match crate::state::persistence::checkpoint::create_checkpoint(
                &path,
                label,
                &state.session,
                &state.instruments,
            ) {
                Ok(id) => {
                    result.push_status(
                        audio.status(),
                        format!("Checkpoint '{}' created ({})", label, id),
                    );
                }
                Err(e) => {
                    result.push_status(audio.status(), format!("Checkpoint failed: {}", e));
                }
            }
        }
        SessionAction::RestoreCheckpoint(checkpoint_id) => {
            let path = state.project.path.clone().unwrap_or_else(default_rack_path);
            match crate::state::persistence::checkpoint::restore_checkpoint(&path, *checkpoint_id) {
                Ok((session, instruments)) => {
                    state.session = session;
                    state.instruments = instruments;
                    state.instruments.rebuild_index();
                    state.undo_history.clear();
                    result.audio_effects = AudioEffect::all();
                    result.push_status(audio.status(), "Checkpoint restored");
                }
                Err(e) => {
                    result.push_status(audio.status(), format!("Restore failed: {}", e));
                }
            }
        }
        SessionAction::DeleteCheckpoint(checkpoint_id) => {
            let path = state.project.path.clone().unwrap_or_else(default_rack_path);
            match crate::state::persistence::checkpoint::delete_checkpoint(&path, *checkpoint_id) {
                Ok(()) => {
                    result.push_status(audio.status(), "Checkpoint deleted");
                }
                Err(e) => {
                    result.push_status(audio.status(), format!("Delete failed: {}", e));
                }
            }
        }
    }

    result
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use crate::state::AppState;
    use imbolc_audio::AudioHandle;
    use imbolc_types::state::music::{Key, Scale};
    use imbolc_types::state::session::MusicalSettings;

    fn setup() -> (AppState, AudioHandle, Sender<IoFeedback>) {
        let (io_tx, _io_rx) = std::sync::mpsc::channel();
        (AppState::new(), AudioHandle::new(), io_tx)
    }

    // -----------------------------------------------------------------------
    // NewProject
    // -----------------------------------------------------------------------

    #[test]
    fn new_project_resets_instruments() {
        let (mut state, mut audio, io_tx) = setup();
        state.add_instrument(crate::state::SourceType::Saw);
        assert!(!state.instruments.instruments.is_empty());

        dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        assert!(state.instruments.instruments.is_empty());
    }

    #[test]
    fn new_project_clears_undo_history() {
        let (mut state, mut audio, io_tx) = setup();
        // Push something so undo is non-empty
        use crate::state::undo::UndoScope;
        state.undo_history.push_scoped(
            UndoScope::Instruments,
            &state.session.clone(),
            &state.instruments.clone(),
        );
        assert!(state.undo_history.can_undo());

        dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        assert!(!state.undo_history.can_undo());
    }

    #[test]
    fn new_project_clears_dirty_flag() {
        let (mut state, mut audio, io_tx) = setup();
        state.project.dirty = true;

        dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        assert!(!state.project.dirty);
    }

    #[test]
    fn new_project_clears_project_path() {
        let (mut state, mut audio, io_tx) = setup();
        state.project.path = Some(PathBuf::from("/some/path.sqlite"));

        dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        assert!(state.project.path.is_none());
    }

    #[test]
    fn new_project_produces_all_audio_effects() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        let all = AudioEffect::all();
        for effect in &all {
            assert!(
                result.audio_effects.contains(effect),
                "Missing audio effect: {:?}",
                effect
            );
        }
    }

    #[test]
    fn new_project_sets_project_name_untitled() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        assert_eq!(result.project_name, Some("untitled".to_string()));
    }

    #[test]
    fn new_project_emits_nav_intents() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(&SessionAction::NewProject, &mut state, &mut audio, &io_tx);

        let nav_dbg: Vec<String> = result.nav.iter().map(|n| format!("{:?}", n)).collect();
        assert!(nav_dbg
            .iter()
            .any(|s| s.contains("ConditionalPop") && s.contains("Confirm")));
        assert!(nav_dbg
            .iter()
            .any(|s| s.contains("ConditionalPop") && s.contains("ProjectBrowser")));
        assert!(nav_dbg
            .iter()
            .any(|s| s.contains("SwitchTo") && s.contains("Add")));
    }

    // -----------------------------------------------------------------------
    // UpdateSession
    // -----------------------------------------------------------------------

    #[test]
    fn update_session_changes_bpm() {
        let (mut state, mut audio, io_tx) = setup();
        assert_eq!(state.session.bpm, 120);

        let settings = MusicalSettings {
            bpm: 140,
            ..MusicalSettings::default()
        };

        dispatch_session(
            &SessionAction::UpdateSession(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert_eq!(state.session.bpm, 140);
    }

    #[test]
    fn update_session_changes_key_and_scale() {
        let (mut state, mut audio, io_tx) = setup();

        let settings = MusicalSettings {
            key: Key::Fs,
            scale: Scale::Dorian,
            ..MusicalSettings::default()
        };

        dispatch_session(
            &SessionAction::UpdateSession(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert_eq!(state.session.key, Key::Fs);
        assert_eq!(state.session.scale, Scale::Dorian);
    }

    #[test]
    fn update_session_produces_rebuild_and_piano_roll_effects() {
        let (mut state, mut audio, io_tx) = setup();

        let settings = MusicalSettings {
            bpm: 90,
            ..MusicalSettings::default()
        };

        let result = dispatch_session(
            &SessionAction::UpdateSession(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
    }

    #[test]
    fn update_session_emits_pop_or_switch_nav() {
        let (mut state, mut audio, io_tx) = setup();

        let settings = MusicalSettings::default();

        let result = dispatch_session(
            &SessionAction::UpdateSession(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        let nav_dbg: Vec<String> = result.nav.iter().map(|n| format!("{:?}", n)).collect();
        assert!(nav_dbg
            .iter()
            .any(|s| s.contains("PopOrSwitchTo") && s.contains("Track")));
    }

    // -----------------------------------------------------------------------
    // UpdateSessionLive
    // -----------------------------------------------------------------------

    #[test]
    fn update_session_live_changes_bpm() {
        let (mut state, mut audio, io_tx) = setup();

        let settings = MusicalSettings {
            bpm: 80,
            ..MusicalSettings::default()
        };

        dispatch_session(
            &SessionAction::UpdateSessionLive(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert_eq!(state.session.bpm, 80);
    }

    #[test]
    fn update_session_live_produces_rebuild_and_piano_roll_effects() {
        let (mut state, mut audio, io_tx) = setup();

        let settings = MusicalSettings {
            bpm: 200,
            ..MusicalSettings::default()
        };

        let result = dispatch_session(
            &SessionAction::UpdateSessionLive(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
        assert!(result.audio_effects.contains(&AudioEffect::UpdatePianoRoll));
    }

    #[test]
    fn update_session_live_has_no_nav_intent() {
        let (mut state, mut audio, io_tx) = setup();

        let settings = MusicalSettings::default();

        let result = dispatch_session(
            &SessionAction::UpdateSessionLive(settings),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!(result.nav.is_empty());
    }

    // -----------------------------------------------------------------------
    // ToggleMasterMute
    // -----------------------------------------------------------------------

    #[test]
    fn toggle_master_mute_toggles_flag() {
        let (mut state, mut audio, io_tx) = setup();
        assert!(!state.session.mixer.master_mute);

        dispatch_session(
            &SessionAction::ToggleMasterMute,
            &mut state,
            &mut audio,
            &io_tx,
        );
        assert!(state.session.mixer.master_mute);

        dispatch_session(
            &SessionAction::ToggleMasterMute,
            &mut state,
            &mut audio,
            &io_tx,
        );
        assert!(!state.session.mixer.master_mute);
    }

    #[test]
    fn toggle_master_mute_produces_rebuild_and_mixer_effects() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(
            &SessionAction::ToggleMasterMute,
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
        assert!(result
            .audio_effects
            .contains(&AudioEffect::UpdateMixerParams));
    }

    // -----------------------------------------------------------------------
    // CycleTheme
    // -----------------------------------------------------------------------

    #[test]
    fn cycle_theme_dark_to_light() {
        let (mut state, mut audio, io_tx) = setup();
        assert_eq!(state.session.theme.name, "Dark");

        dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);

        assert_eq!(state.session.theme.name, "Light");
    }

    #[test]
    fn cycle_theme_light_to_high_contrast() {
        let (mut state, mut audio, io_tx) = setup();
        // Set to Light first
        dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);
        assert_eq!(state.session.theme.name, "Light");

        dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);

        assert_eq!(state.session.theme.name, "High Contrast");
    }

    #[test]
    fn cycle_theme_wraps_to_dark() {
        let (mut state, mut audio, io_tx) = setup();
        // Cycle through all three
        dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);
        dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);
        assert_eq!(state.session.theme.name, "High Contrast");

        dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);

        assert_eq!(state.session.theme.name, "Dark");
    }

    #[test]
    fn cycle_theme_produces_status_message() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);

        assert!(!result.status.is_empty());
        assert!(result.status[0].message.contains("Theme:"));
    }

    #[test]
    fn cycle_theme_no_audio_effects() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(&SessionAction::CycleTheme, &mut state, &mut audio, &io_tx);

        assert!(result.audio_effects.is_empty());
    }

    // -----------------------------------------------------------------------
    // Save
    // -----------------------------------------------------------------------

    #[test]
    fn save_sets_in_progress_and_clears_error() {
        let (mut state, mut audio, io_tx) = setup();
        state.io.last_io_error = Some("previous error".to_string());

        dispatch_session(&SessionAction::Save, &mut state, &mut audio, &io_tx);

        assert!(state.io.save_in_progress);
        assert!(state.io.last_io_error.is_none());
    }

    // -----------------------------------------------------------------------
    // AdjustHumanizeVelocity
    // -----------------------------------------------------------------------

    #[test]
    fn humanize_velocity_increases() {
        let (mut state, mut audio, io_tx) = setup();
        assert_eq!(state.session.humanize.velocity, 0.0);

        dispatch_session(
            &SessionAction::AdjustHumanizeVelocity(0.3),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!((state.session.humanize.velocity - 0.3).abs() < f32::EPSILON);
    }

    #[test]
    fn humanize_velocity_clamps_to_one() {
        let (mut state, mut audio, io_tx) = setup();

        dispatch_session(
            &SessionAction::AdjustHumanizeVelocity(1.5),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!((state.session.humanize.velocity - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn humanize_velocity_clamps_to_zero() {
        let (mut state, mut audio, io_tx) = setup();
        state.session.humanize.velocity = 0.2;

        dispatch_session(
            &SessionAction::AdjustHumanizeVelocity(-0.5),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!((state.session.humanize.velocity - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn humanize_velocity_produces_rebuild_session() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(
            &SessionAction::AdjustHumanizeVelocity(0.1),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
    }

    // -----------------------------------------------------------------------
    // AdjustHumanizeTiming
    // -----------------------------------------------------------------------

    #[test]
    fn humanize_timing_increases() {
        let (mut state, mut audio, io_tx) = setup();
        assert_eq!(state.session.humanize.timing, 0.0);

        dispatch_session(
            &SessionAction::AdjustHumanizeTiming(0.4),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!((state.session.humanize.timing - 0.4).abs() < f32::EPSILON);
    }

    #[test]
    fn humanize_timing_clamps_to_one() {
        let (mut state, mut audio, io_tx) = setup();

        dispatch_session(
            &SessionAction::AdjustHumanizeTiming(2.0),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!((state.session.humanize.timing - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn humanize_timing_clamps_to_zero() {
        let (mut state, mut audio, io_tx) = setup();
        state.session.humanize.timing = 0.1;

        dispatch_session(
            &SessionAction::AdjustHumanizeTiming(-0.5),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!((state.session.humanize.timing - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn humanize_timing_produces_rebuild_session() {
        let (mut state, mut audio, io_tx) = setup();

        let result = dispatch_session(
            &SessionAction::AdjustHumanizeTiming(0.1),
            &mut state,
            &mut audio,
            &io_tx,
        );

        assert!(result.audio_effects.contains(&AudioEffect::RebuildSession));
    }
}
