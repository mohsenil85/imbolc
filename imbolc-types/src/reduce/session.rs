use crate::{SessionAction, SessionState, TrackState};

pub(super) fn reduce(
    action: &SessionAction,
    session: &mut SessionState,
    _instruments: &mut TrackState,
) -> bool {
    match action {
        SessionAction::NewProject => false,
        SessionAction::SetSession(ref settings) => {
            session.apply_musical_settings(settings);
            true
        }
        SessionAction::SetSessionLive(ref settings) => {
            session.apply_musical_settings(settings);
            true
        }
        SessionAction::AdjustHumanizeVelocity(delta) => {
            session.humanize.velocity = (session.humanize.velocity + delta).clamp(0.0, 1.0);
            true
        }
        SessionAction::AdjustHumanizeTiming(delta) => {
            session.humanize.timing = (session.humanize.timing + delta).clamp(0.0, 1.0);
            true
        }
        SessionAction::ToggleMasterMute => {
            session.mixer.master_mute = !session.mixer.master_mute;
            true
        }
        SessionAction::NextTheme => {
            use crate::state::Theme;
            let current_name = &session.theme.name;
            session.theme = match current_name.as_str() {
                "Minimal" => Theme::minimal_light(),
                "Minimal Light" => Theme::dark(),
                "Dark" => Theme::light(),
                "Light" => Theme::high_contrast(),
                _ => Theme::minimal(),
            };
            true
        }
        SessionAction::ImportVstPlugin(ref path, kind) => {
            use crate::state::vst::VstPlugin;
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| "VST Plugin".to_string());
            let plugin = VstPlugin {
                id: crate::VstPluginId::new(0),
                name,
                plugin_path: path.clone(),
                kind: *kind,
                params: vec![],
            };
            session.vst_plugins.add(plugin);
            true
        }
        // RequestFileBrowser: navigation only
        SessionAction::RequestFileBrowser(_) => true,
        // Runtime waveform import/editing is handled by dispatch/session.rs.
        SessionAction::LoadWaveformFileResult(_)
        | SessionAction::DeleteWaveformSelection { .. }
        | SessionAction::TrimWaveformSelection { .. }
        | SessionAction::RenameWaveformClip(_)
        | SessionAction::CreatePitchedSamplerFromWaveformSelection { .. }
        | SessionAction::CreateTimeStretchFromWaveformSelection { .. }
        | SessionAction::OpenWaveformSelectionInChopper { .. } => false,
        // File I/O actions: not reducible
        SessionAction::Save
        | SessionAction::SaveAs(_)
        | SessionAction::Load
        | SessionAction::LoadFrom(_)
        | SessionAction::ImportCustomSynthDef(_)
        | SessionAction::CreateCheckpoint(_)
        | SessionAction::RestoreCheckpoint(_)
        | SessionAction::DeleteCheckpoint(_) => false,
    }
}
