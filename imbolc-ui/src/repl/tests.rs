use super::*;
use imbolc_types::{DomainAction, NavAction, PaneId};

fn test_setup() -> (LocalDispatcher, AudioHandle) {
    // SAFETY: tests run with --test-threads=1 or accept env-var race; this is standard test setup.
    unsafe { std::env::set_var("IMBOLC_NO_AUDIO", "1") };
    let (io_tx, _io_rx) = std::sync::mpsc::channel();
    let config = crate::config::Config::load();
    let state = AppState::new_with_defaults(config.defaults());
    let audio = AudioHandle::new();
    let dispatcher = LocalDispatcher::new(state, io_tx);
    (dispatcher, audio)
}

fn exec(
    input: &str,
    dispatcher: &mut LocalDispatcher,
    audio: &mut AudioHandle,
) -> Result<ReplResult, String> {
    parse_and_execute(input, dispatcher, audio)
}

// ---------------------------------------------------------------------------
// 1–4: replacements for the old tmux e2e tests
// ---------------------------------------------------------------------------

#[test]
fn test_initial_state() {
    let (mut d, mut a) = test_setup();

    // No instruments on fresh state
    match exec("show tracks", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => assert!(
            text.contains("No tracks"),
            "expected 'No tracks', got: {text}"
        ),
        other => panic!("expected Output, got: {other:?}"),
    }

    // Transport defaults
    match exec("show transport", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(text.contains("STOPPED"), "expected STOPPED in: {text}");
            assert!(text.contains("BPM: 120"), "expected BPM: 120 in: {text}");
        }
        other => panic!("expected Output, got: {other:?}"),
    }
}

#[test]
fn test_quit_command() {
    let (mut d, mut a) = test_setup();
    assert!(matches!(exec("quit", &mut d, &mut a), Ok(ReplResult::Quit)));
    assert!(matches!(exec("exit", &mut d, &mut a), Ok(ReplResult::Quit)));
}

#[test]
fn test_add_track() {
    let (mut d, mut a) = test_setup();

    exec("track add saw", &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.tracks.len(), 1);
    assert_eq!(d.state().tracks.tracks[0].source.short_name(), "saw");

    match exec("show tracks", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => assert!(text.contains("saw"), "expected saw in: {text}"),
        other => panic!("expected Output, got: {other:?}"),
    }
}

#[test]
fn test_session_settings() {
    let (mut d, mut a) = test_setup();

    exec("set bpm 140", &mut d, &mut a).unwrap();
    exec("set key D", &mut d, &mut a).unwrap();

    assert_eq!(d.state().session.bpm, 140);
    assert_eq!(d.state().session.key.name(), "D");

    match exec("show session", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(text.contains("BPM: 140"), "expected BPM: 140 in: {text}");
            assert!(text.contains("D"), "expected key D in: {text}");
        }
        other => panic!("expected Output, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 5–11: new coverage
// ---------------------------------------------------------------------------

#[test]
fn test_track_lifecycle() {
    let (mut d, mut a) = test_setup();

    exec("track add saw", &mut d, &mut a).unwrap();
    exec("track add sin", &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.tracks.len(), 2);

    exec("track select 0", &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.selected, Some(0));

    let id = d.state().tracks.tracks[0].id;
    exec(&format!("track delete {}", id), &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.tracks.len(), 1);
}

#[test]
fn test_mixer_operations() {
    let (mut d, mut a) = test_setup();

    exec("track add saw", &mut d, &mut a).unwrap();
    // Mixer selection starts at channel 0 (first track)
    exec("mixer toggle-mute", &mut d, &mut a).unwrap();
    assert!(
        d.state().tracks.tracks[0].channel_strip.mute,
        "track should be muted"
    );

    exec("mixer toggle-mute", &mut d, &mut a).unwrap();
    assert!(
        !d.state().tracks.tracks[0].channel_strip.mute,
        "track should be unmuted"
    );
}

#[test]
fn test_undo_redo() {
    let (mut d, mut a) = test_setup();

    exec("track add saw", &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.tracks.len(), 1);

    exec("undo", &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.tracks.len(), 0);

    exec("redo", &mut d, &mut a).unwrap();
    assert_eq!(d.state().tracks.tracks.len(), 1);
}

#[test]
fn test_show_all_targets() {
    let (mut d, mut a) = test_setup();

    // Add a track + select it so sequencer/notes have something to display
    exec("track add saw", &mut d, &mut a).unwrap();

    let targets = [
        "tracks",
        "transport",
        "mixer",
        "buses",
        "arrangement",
        "automation",
        "generative",
        "session",
        "server",
        "midi",
    ];

    for target in &targets {
        let result = exec(&format!("show {target}"), &mut d, &mut a);
        match result {
            Ok(ReplResult::Output(text)) => {
                assert!(!text.is_empty(), "show {target} returned empty output");
            }
            other => panic!("show {target} expected Output, got: {other:?}"),
        }
    }
}

#[test]
fn test_error_cases() {
    let (mut d, mut a) = test_setup();

    // Unknown command goes to registry which returns an error
    assert!(exec("nonexistent", &mut d, &mut a).is_err());

    // Missing args
    assert!(exec("set", &mut d, &mut a).is_err());
    let err = exec("set bpm", &mut d, &mut a).unwrap_err();
    assert!(
        err.contains("bpm") || err.contains("set"),
        "error should mention the command: {err}"
    );

    // Invalid args
    assert!(exec("set bpm notanumber", &mut d, &mut a).is_err());

    // show with no target
    assert!(exec("show", &mut d, &mut a).is_err());

    // show with unknown target
    assert!(exec("show nonexistent", &mut d, &mut a).is_err());
}

#[test]
fn test_help_system() {
    let (mut d, mut a) = test_setup();

    match exec("help", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(
                text.contains("Command groups"),
                "help should list command groups: {text}"
            );
        }
        other => panic!("expected Output, got: {other:?}"),
    }

    match exec("help track", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(text.contains("add"), "help track should list 'add': {text}");
        }
        other => panic!("expected Output, got: {other:?}"),
    }

    match exec("help nonexistent", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(
                text.contains("Unknown group"),
                "help nonexistent should say unknown: {text}"
            );
        }
        other => panic!("expected Output, got: {other:?}"),
    }
}

#[test]
fn test_status_command() {
    let (mut d, mut a) = test_setup();

    exec("track add saw", &mut d, &mut a).unwrap();
    exec("track add sin", &mut d, &mut a).unwrap();
    exec("set bpm 160", &mut d, &mut a).unwrap();

    match exec("status", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(text.contains("Tracks: 2"), "expected track count: {text}");
            assert!(text.contains("BPM: 160"), "expected bpm: {text}");
            assert!(text.contains("STOPPED"), "expected stopped: {text}");
        }
        other => panic!("expected Output, got: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// Trailing token rejection
// ---------------------------------------------------------------------------

#[test]
fn test_trailing_tokens_rejected() {
    let (mut d, mut a) = test_setup();

    // Registry commands should reject trailing tokens
    let err = exec("track add saw garbage", &mut d, &mut a).unwrap_err();
    assert!(
        err.contains("unexpected trailing"),
        "expected trailing error, got: {err}"
    );

    // Meta commands should reject trailing tokens
    assert!(exec("undo extra", &mut d, &mut a).is_err());
    assert!(exec("redo extra", &mut d, &mut a).is_err());
    assert!(exec("status extra", &mut d, &mut a).is_err());
    assert!(exec("quit extra", &mut d, &mut a).is_err());
    assert!(exec("save path extra", &mut d, &mut a).is_err());
    assert!(exec("load path extra", &mut d, &mut a).is_err());
}

// ---------------------------------------------------------------------------
// Quoted string arguments
// ---------------------------------------------------------------------------

#[test]
fn test_quoted_args() {
    let (mut d, mut a) = test_setup();

    // Quoted strings should work for string arguments
    let result = exec(r#"session create-checkpoint "pre-mix v2""#, &mut d, &mut a);
    assert!(
        result.is_ok(),
        "quoted checkpoint name should work: {:?}",
        result.err()
    );
}

#[test]
fn test_unclosed_quote_error() {
    let (mut d, mut a) = test_setup();

    let err = exec(r#"session create-checkpoint "unclosed"#, &mut d, &mut a).unwrap_err();
    assert!(
        err.contains("unclosed quote"),
        "expected unclosed quote error, got: {err}"
    );
}

// ---------------------------------------------------------------------------
// Compile-time REPL exhaustiveness for DomainAction
// ---------------------------------------------------------------------------

/// Map every DomainAction variant to its expected REPL group name (or None).
///
/// The exhaustive match is the real test: adding a new DomainAction variant
/// without updating this function causes a compile error, forcing you to
/// decide whether it gets a REPL group or is explicitly excluded.
fn expected_repl_group(action: &DomainAction) -> Option<&'static str> {
    match action {
        DomainAction::Track(..) => Some("track"),
        DomainAction::Mixer(..) => Some("mixer"),
        DomainAction::PianoRoll(..) => Some("piano-roll"),
        DomainAction::Arrangement(..) => Some("arrangement"),
        DomainAction::Server(..) => Some("server"),
        DomainAction::Session(..) => Some("session"),
        DomainAction::Sequencer(..) => Some("sequencer"),
        DomainAction::SampleSlicer(..) => Some("slicer"),
        DomainAction::Automation(..) => Some("automation"),
        DomainAction::Midi(..) => Some("midi"),
        DomainAction::Bus(..) => Some("bus"),
        DomainAction::Group(..) => Some("layer-group"),
        DomainAction::Master(..) => Some("master"),
        DomainAction::VstParam(..) => Some("vst-param"),
        DomainAction::Click(..) => Some("click"),
        DomainAction::Tuner(..) => Some("tuner"),
        DomainAction::Generative(..) => Some("generative"),
        // Intentionally excluded from REPL:
        DomainAction::Tag(..) => None, // TODO: add REPL commands
        DomainAction::AudioFeedback(..) => None, // internal audio thread feedback
        DomainAction::Undo | DomainAction::Redo => None, // top-level commands in parse_command()
    }
}

#[test]
fn repl_covers_all_domain_actions() {
    let groups = registry::group_names();

    // Collect the unique group names referenced in the exhaustive match.
    // We use a dummy action per variant just to exercise every arm.
    let dummy_actions: Vec<DomainAction> = vec![
        DomainAction::Track(TrackAction::SelectNext),
        DomainAction::Mixer(MixerAction::ToggleMute),
        DomainAction::PianoRoll(PianoRollAction::ToggleNote {
            pitch: 60,
            tick: 0,
            duration: 480,
            velocity: 100,
            track: 0,
        }),
        DomainAction::Arrangement(ArrangementAction::TogglePlayMode),
        DomainAction::Server(ServerAction::Connect),
        DomainAction::Session(SessionAction::Save),
        DomainAction::Sequencer(SequencerAction::ToggleStep(0, 0)),
        DomainAction::SampleSlicer(SampleSlicerAction::LoadSample),
        DomainAction::Automation(AutomationAction::AddLane(AutomationTarget::Global(
            GlobalParameter::Bpm,
        ))),
        DomainAction::Midi(MidiAction::ConnectPort(0)),
        DomainAction::Bus(BusAction::Add),
        DomainAction::Group(GroupAction::AddEffect(GroupId::new(0), EffectType::Delay)),
        DomainAction::Master(MasterAction::AddEffect(EffectType::Delay)),
        DomainAction::VstParam(VstParamAction::SetParam(
            TrackId::new(0),
            VstTarget::Source,
            0,
            0.0,
        )),
        DomainAction::Click(ClickAction::Toggle),
        DomainAction::Tuner(TunerAction::PlayTone(440.0)),
        DomainAction::Generative(GenerativeAction::ToggleEnabled),
        DomainAction::Tag(TagAction::CreateTag(String::new())),
        DomainAction::AudioFeedback(imbolc_types::AudioFeedback::PendingBufferFreed),
        DomainAction::Undo,
        DomainAction::Redo,
    ];

    for action in &dummy_actions {
        if let Some(group) = expected_repl_group(action) {
            assert!(
                groups.contains(&group),
                "DomainAction::{:?} maps to REPL group '{}' which doesn't exist in registry. \
                 Available groups: {:?}",
                std::mem::discriminant(action),
                group,
                groups,
            );
        }
    }
}

// ---------------------------------------------------------------------------
// View command
// ---------------------------------------------------------------------------

fn parse(input: &str) -> Result<CommandResult, String> {
    let state = AppState::default();
    parse_command(input, &state)
}

#[test]
fn test_view_mixer() {
    match parse("view mixer").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::Mixer)) => {}
        other => panic!("expected Nav(SwitchPane(Mixer)), got: {other:?}"),
    }
}

#[test]
fn test_view_piano_roll_hyphen() {
    match parse("view piano-roll").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::PianoRoll)) => {}
        other => panic!("expected Nav(SwitchPane(PianoRoll)), got: {other:?}"),
    }
}

#[test]
fn test_view_piano_roll_underscore() {
    match parse("view piano_roll").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::PianoRoll)) => {}
        other => panic!("expected Nav(SwitchPane(PianoRoll)), got: {other:?}"),
    }
}

#[test]
fn test_view_short_alias_pr() {
    match parse("view pr").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::PianoRoll)) => {}
        other => panic!("expected Nav(SwitchPane(PianoRoll)), got: {other:?}"),
    }
}

#[test]
fn test_view_short_alias_mix() {
    match parse("view mix").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::Mixer)) => {}
        other => panic!("expected Nav(SwitchPane(Mixer)), got: {other:?}"),
    }
}

#[test]
fn test_view_short_alias_auto() {
    match parse("view auto").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::Automation)) => {}
        other => panic!("expected Nav(SwitchPane(Automation)), got: {other:?}"),
    }
}

#[test]
fn test_view_track() {
    match parse("view track").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::Track)) => {}
        other => panic!("expected Nav(SwitchPane(Track)), got: {other:?}"),
    }
}

#[test]
fn test_view_instrument() {
    match parse("view instrument").unwrap() {
        CommandResult::Nav(NavAction::SwitchPane(PaneId::InstrumentEdit)) => {}
        other => panic!("expected Nav(SwitchPane(InstrumentEdit)), got: {other:?}"),
    }
}

#[test]
fn test_view_no_arg() {
    assert!(parse("view").is_err());
}

#[test]
fn test_view_unknown() {
    assert!(parse("view nonexistent").is_err());
}

#[test]
fn test_view_non_switchable() {
    assert!(parse("view add").is_err());
}

#[test]
fn test_view_trailing_rejected() {
    assert!(parse("view mixer extra").is_err());
}
