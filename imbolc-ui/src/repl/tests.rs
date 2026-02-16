use super::*;

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
    match exec("show instruments", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => assert!(
            text.contains("No instruments"),
            "expected 'No instruments', got: {text}"
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
fn test_add_instrument() {
    let (mut d, mut a) = test_setup();

    exec("instrument add saw", &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.instruments.len(), 1);
    assert_eq!(
        d.state().instruments.instruments[0].source.short_name(),
        "saw"
    );

    match exec("show instruments", &mut d, &mut a).unwrap() {
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
fn test_instrument_lifecycle() {
    let (mut d, mut a) = test_setup();

    exec("instrument add saw", &mut d, &mut a).unwrap();
    exec("instrument add sin", &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.instruments.len(), 2);

    exec("instrument select 0", &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.selected, Some(0));

    let id = d.state().instruments.instruments[0].id;
    exec(&format!("instrument delete {}", id), &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.instruments.len(), 1);
}

#[test]
fn test_mixer_operations() {
    let (mut d, mut a) = test_setup();

    exec("instrument add saw", &mut d, &mut a).unwrap();
    // Mixer selection starts at channel 0 (first instrument)
    exec("mixer toggle-mute", &mut d, &mut a).unwrap();
    assert!(
        d.state().instruments.instruments[0].mixer.mute,
        "instrument should be muted"
    );

    exec("mixer toggle-mute", &mut d, &mut a).unwrap();
    assert!(
        !d.state().instruments.instruments[0].mixer.mute,
        "instrument should be unmuted"
    );
}

#[test]
fn test_undo_redo() {
    let (mut d, mut a) = test_setup();

    exec("instrument add saw", &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.instruments.len(), 1);

    exec("undo", &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.instruments.len(), 0);

    exec("redo", &mut d, &mut a).unwrap();
    assert_eq!(d.state().instruments.instruments.len(), 1);
}

#[test]
fn test_show_all_targets() {
    let (mut d, mut a) = test_setup();

    // Add an instrument + select it so sequencer/notes have something to display
    exec("instrument add saw", &mut d, &mut a).unwrap();

    let targets = [
        "instruments",
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

    match exec("help instrument", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(
                text.contains("add"),
                "help instrument should list 'add': {text}"
            );
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

    exec("instrument add saw", &mut d, &mut a).unwrap();
    exec("instrument add sin", &mut d, &mut a).unwrap();
    exec("set bpm 160", &mut d, &mut a).unwrap();

    match exec("status", &mut d, &mut a).unwrap() {
        ReplResult::Output(text) => {
            assert!(
                text.contains("Instruments: 2"),
                "expected instrument count: {text}"
            );
            assert!(text.contains("BPM: 160"), "expected bpm: {text}");
            assert!(text.contains("STOPPED"), "expected stopped: {text}");
        }
        other => panic!("expected Output, got: {other:?}"),
    }
}
