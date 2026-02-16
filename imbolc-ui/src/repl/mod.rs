//! Accessible REPL mode — full DAW control via text commands over plain stdin/stdout.
//!
//! Launched via `imbolc-ui --repl`. No TUI rendering, no alternate screen buffer.

pub mod completion;
pub mod display;
pub mod macro_def;
pub mod parse;
pub mod registry;

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};

use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::action::IoFeedback;
use crate::audio::AudioHandle;
use crate::config;
use crate::dispatch::LocalDispatcher;
use crate::setup;
use crate::state::{self, AppState};
use imbolc_types::*;

use completion::ReplHelper;

#[derive(Debug)]
enum ReplResult {
    /// A read-only query produced output text.
    Output(String),
    /// An action was dispatched; apply audio effects and print confirmation.
    Dispatched(DomainAction, DispatchResult),
    /// User wants to quit.
    Quit,
}

#[cfg(test)]
mod tests;

pub fn run(project_arg: Option<String>) -> ! {
    let (io_tx, io_rx) = mpsc::channel::<IoFeedback>();
    let config = config::Config::load();
    let mut state = AppState::new_with_defaults(config.defaults());

    // Load project if specified
    if let Some(ref path_str) = project_arg {
        let load_path = PathBuf::from(path_str);
        if load_path.exists() {
            if let Ok((session, instruments)) = state::persistence::load_project(&load_path) {
                state.session = session;
                state.instruments = instruments;
                state.instruments.rebuild_index();
                state.project.path = Some(load_path);
                state.project.dirty = false;
            } else {
                eprintln!("Warning: failed to load project from {}", path_str);
            }
        } else {
            state.project.path = Some(load_path);
        }
    }

    let mut audio = AudioHandle::new();
    audio.sync_state(&state);

    let mut dispatcher = LocalDispatcher::new(state, io_tx);

    // Auto-start SuperCollider
    let _startup_events = setup::auto_start_sc(&mut audio);

    // Set up rustyline editor with tab completion
    let rl_config = rustyline::Config::builder().auto_add_history(true).build();
    let mut rl = Editor::with_config(rl_config).expect("failed to create readline editor");
    rl.set_helper(Some(ReplHelper));

    println!("imbolc REPL — type 'help' for commands, 'quit' to exit");

    // Print initial state summary
    if !dispatcher.state().instruments.instruments.is_empty() {
        println!(
            "{} instruments loaded",
            dispatcher.state().instruments.instruments.len()
        );
    }

    loop {
        // Drain any pending feedback before prompting
        drain_io_feedback(&mut dispatcher, &io_rx);
        drain_audio_feedback(&mut dispatcher, &mut audio);

        let readline = rl.readline("imbolc> ");
        match readline {
            Ok(line) => {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }

                match parse_and_execute(line, &mut dispatcher, &mut audio) {
                    Ok(ReplResult::Output(text)) => {
                        println!("{}", text);
                    }
                    Ok(ReplResult::Dispatched(action, result)) => {
                        // Apply audio effects
                        audio.apply_effects(
                            dispatcher.state(),
                            &result.audio_effects,
                            result.needs_full_sync,
                        );

                        print_confirmation(&action, &result);

                        if result.quit {
                            std::process::exit(0);
                        }
                    }
                    Ok(ReplResult::Quit) => {
                        println!("Goodbye.");
                        std::process::exit(0);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted | ReadlineError::Eof) => {
                println!("Goodbye.");
                std::process::exit(0);
            }
            Err(err) => {
                eprintln!("Readline error: {}", err);
                std::process::exit(1);
            }
        }
    }
}

fn parse_and_execute(
    input: &str,
    dispatcher: &mut LocalDispatcher,
    audio: &mut AudioHandle,
) -> Result<ReplResult, String> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return Err("empty command".to_string());
    }

    // Meta commands
    match parts[0] {
        "quit" | "exit" => return Ok(ReplResult::Quit),
        "help" => return Ok(ReplResult::Output(format_help(&parts[1..]))),
        "show" => return handle_show(&parts[1..], dispatcher.state()),
        "set" => return handle_set(&parts[1..], dispatcher, audio),
        "undo" => {
            let result = dispatcher.dispatch_domain(&DomainAction::Undo, audio);
            return Ok(ReplResult::Dispatched(DomainAction::Undo, result));
        }
        "redo" => {
            let result = dispatcher.dispatch_domain(&DomainAction::Redo, audio);
            return Ok(ReplResult::Dispatched(DomainAction::Redo, result));
        }
        "save" => {
            return if parts.len() > 1 {
                let path = PathBuf::from(parts[1]);
                let action = DomainAction::Session(SessionAction::SaveAs(path));
                let result = dispatcher.dispatch_domain(&action, audio);
                Ok(ReplResult::Dispatched(action, result))
            } else {
                let action = DomainAction::Session(SessionAction::Save);
                let result = dispatcher.dispatch_domain(&action, audio);
                Ok(ReplResult::Dispatched(action, result))
            };
        }
        "load" => {
            return if parts.len() > 1 {
                let path = PathBuf::from(parts[1]);
                let action = DomainAction::Session(SessionAction::LoadFrom(path));
                let result = dispatcher.dispatch_domain(&action, audio);
                Ok(ReplResult::Dispatched(action, result))
            } else {
                let action = DomainAction::Session(SessionAction::Load);
                let result = dispatcher.dispatch_domain(&action, audio);
                Ok(ReplResult::Dispatched(action, result))
            };
        }
        "status" => {
            let state = dispatcher.state();
            let server = format!("{:?}", state.audio.server_status);
            let playing = if state.audio.playing {
                "PLAYING"
            } else {
                "STOPPED"
            };
            let n_inst = state.instruments.instruments.len();
            return Ok(ReplResult::Output(format!(
                "Server: {}  Transport: {}  Instruments: {}  BPM: {}",
                server, playing, n_inst, state.session.bpm,
            )));
        }
        _ => {}
    }

    // Try the generated registry parser
    let action = registry::parse_action(input)?;
    let result = dispatcher.dispatch_domain(&action, audio);
    Ok(ReplResult::Dispatched(action, result))
}

fn handle_show(args: &[&str], state: &AppState) -> Result<ReplResult, String> {
    if args.is_empty() {
        return Err(
            "show what? Try: show instruments, show transport, show mixer, ...".to_string(),
        );
    }

    let output = match args[0] {
        "instruments" => display::format_instrument_list(state),
        "instrument" => {
            if args.len() < 2 {
                return Err("show instrument <id>".to_string());
            }
            let id: u32 = args[1]
                .parse()
                .map_err(|_| format!("invalid instrument id: {}", args[1]))?;
            display::format_instrument_detail(state, InstrumentId::new(id))
        }
        "transport" => display::format_transport(state),
        "mixer" => display::format_mixer(state),
        "notes" => {
            if args.len() < 2 {
                return Err("show notes <track_index>".to_string());
            }
            let idx: usize = args[1]
                .parse()
                .map_err(|_| format!("invalid track index: {}", args[1]))?;
            display::format_notes(state, idx)
        }
        "effects" => {
            if args.len() < 2 {
                return Err("show effects <instrument_id>".to_string());
            }
            let id: u32 = args[1]
                .parse()
                .map_err(|_| format!("invalid instrument id: {}", args[1]))?;
            display::format_effects(state, InstrumentId::new(id))
        }
        "buses" => display::format_buses(state),
        "arrangement" => display::format_arrangement(state),
        "automation" => display::format_automation(state),
        "generative" => display::format_generative(state),
        "session" => display::format_session(state),
        "sequencer" => display::format_sequencer(state),
        "server" => display::format_server(state),
        "midi" => display::format_midi(state),
        other => return Err(format!("unknown show target: '{}'", other)),
    };

    Ok(ReplResult::Output(output))
}

fn handle_set(
    args: &[&str],
    dispatcher: &mut LocalDispatcher,
    audio: &mut AudioHandle,
) -> Result<ReplResult, String> {
    if args.len() < 2 {
        return Err("set what? Try: set bpm 140, set key C, set scale Minor, set tuning 12-TET, set ji-flavor 5-Limit".to_string());
    }

    let state = dispatcher.state();
    let mut settings = MusicalSettings {
        bpm: state.session.bpm,
        key: state.session.key,
        scale: state.session.scale,
        tuning: state.session.tuning,
        tuning_a4: state.session.tuning_a4,
        ji_flavor: state.session.ji_flavor,
        time_signature: state.session.time_signature,
        snap: state.session.snap,
    };

    match args[0] {
        "bpm" => {
            let bpm: u16 = args[1]
                .parse()
                .map_err(|_| format!("invalid BPM: {}", args[1]))?;
            settings.bpm = bpm;
        }
        "key" => {
            let key = Key::from_name(args[1])
                .ok_or_else(|| format!("unknown key: {} (try: C, C#, D, ...)", args[1]))?;
            settings.key = key;
        }
        "scale" => {
            let scale = Scale::from_name(args[1])
                .ok_or_else(|| format!("unknown scale: {} (try: Major, Minor, ...)", args[1]))?;
            settings.scale = scale;
        }
        "tuning" => {
            let tuning = Tuning::from_name(args[1]).ok_or_else(|| {
                format!(
                    "unknown tuning: {} (try: 12-TET, ScaleJI, ChordJI, ...)",
                    args[1]
                )
            })?;
            settings.tuning = tuning;
        }
        "ji-flavor" => {
            let flavor = JIFlavor::from_name(args[1]).ok_or_else(|| {
                format!(
                    "unknown JI flavor: {} (try: 5-Limit, 7-Limit, Pythagorean)",
                    args[1]
                )
            })?;
            settings.ji_flavor = flavor;
        }
        "tuning-a4" => {
            let freq: f32 = args[1]
                .parse()
                .map_err(|_| format!("invalid frequency: {}", args[1]))?;
            settings.tuning_a4 = freq;
        }
        "snap" => {
            let snap = parse::ReplParseable::parse_repl(args[1])
                .map_err(|_| format!("invalid snap value: {} (try: on, off)", args[1]))?;
            settings.snap = snap;
        }
        other => {
            return Err(format!(
                "unknown setting: '{}' (try: bpm, key, scale, tuning, ji-flavor, tuning-a4, snap)",
                other
            ))
        }
    }

    let action = DomainAction::Session(SessionAction::UpdateSession(settings));
    let result = dispatcher.dispatch_domain(&action, audio);
    Ok(ReplResult::Dispatched(action, result))
}

fn format_help(args: &[&str]) -> String {
    let all_cmds = registry::all_commands();

    if args.is_empty() {
        // Top-level help
        let mut lines = vec![
            "imbolc REPL commands:".to_string(),
            String::new(),
            "Meta commands:".to_string(),
            "  help [group] [command]  Show help".to_string(),
            "  show <target>           Show state (instruments, mixer, transport, ...)".to_string(),
            "  set <param> <value>     Set session parameter (bpm, key, scale, ...)".to_string(),
            "  undo                    Undo last action".to_string(),
            "  redo                    Redo last undone action".to_string(),
            "  save [path]             Save project".to_string(),
            "  load [path]             Load project".to_string(),
            "  status                  Show status summary".to_string(),
            "  quit                    Exit REPL".to_string(),
            String::new(),
            "Command groups:".to_string(),
        ];

        let groups = registry::group_names();
        for group in groups {
            let count = all_cmds.iter().filter(|c| c.group == *group).count();
            lines.push(format!("  {:<20} ({} commands)", group, count));
        }

        lines.push(String::new());
        lines.push("Type 'help <group>' for commands in a group.".to_string());
        lines.join("\n")
    } else if args.len() == 1 {
        // Group help
        let group = args[0];
        let group_cmds: Vec<_> = all_cmds.iter().filter(|c| c.group == group).collect();

        if group_cmds.is_empty() {
            return format!("Unknown group: '{}'. Try: help", group);
        }

        let mut lines = vec![format!("Commands for '{}':", group)];
        for cmd in &group_cmds {
            let args_str = if cmd.args.is_empty() {
                String::new()
            } else {
                format!(" {}", cmd.args.trim())
            };
            lines.push(format!(
                "  {} {}{:<24} {}",
                cmd.group, cmd.name, args_str, cmd.description
            ));
        }
        lines.join("\n")
    } else {
        // Specific command help
        let group = args[0];
        let cmd_name = args[1];
        match all_cmds
            .iter()
            .find(|c| c.group == group && c.name == cmd_name)
        {
            Some(cmd) => {
                let args_str = if cmd.args.is_empty() {
                    String::new()
                } else {
                    format!(" {}", cmd.args.trim())
                };
                format!(
                    "{} {}{}\n  {}",
                    cmd.group, cmd.name, args_str, cmd.description
                )
            }
            None => format!(
                "Unknown command: '{} {}'. Try: help {}",
                group, cmd_name, group
            ),
        }
    }
}

fn print_confirmation(action: &DomainAction, result: &DispatchResult) {
    if !result.status.is_empty() {
        for event in &result.status {
            println!("  [status] {:?}", event);
        }
    }

    // Simple confirmation based on action type
    match action {
        DomainAction::Undo => println!("  Undone."),
        DomainAction::Redo => println!("  Redone."),
        _ => {
            if !result.audio_effects.is_empty() {
                println!("  OK ({} audio effects)", result.audio_effects.len());
            } else {
                println!("  OK");
            }
        }
    }
}

fn drain_io_feedback(dispatcher: &mut LocalDispatcher, io_rx: &Receiver<IoFeedback>) {
    while let Ok(feedback) = io_rx.try_recv() {
        match feedback {
            IoFeedback::SaveComplete { path, result, .. } => match result {
                Ok(name) => {
                    let state = dispatcher.state_mut();
                    state.project.path = Some(path);
                    state.project.dirty = false;
                    println!("  Project saved: {}", name);
                }
                Err(e) => eprintln!("  Save failed: {}", e),
            },
            IoFeedback::LoadComplete { path, result, .. } => match result {
                Ok((session, instruments, name)) => {
                    let state = dispatcher.state_mut();
                    state.session = session;
                    state.instruments = instruments;
                    state.instruments.rebuild_index();
                    state.project.path = Some(path);
                    state.project.dirty = false;
                    println!("  Project loaded: {}", name);
                }
                Err(e) => eprintln!("  Load failed: {}", e),
            },
            IoFeedback::AutosaveComplete { .. } => {}
            IoFeedback::ImportSynthDefComplete { result, .. } => match result {
                Ok((_, name, _)) => println!("  SynthDef imported: {}", name),
                Err(e) => eprintln!("  SynthDef import failed: {}", e),
            },
            IoFeedback::ImportSynthDefLoaded { result, .. } => {
                if let Err(e) = result {
                    eprintln!("  SynthDef load failed: {}", e);
                }
            }
        }
    }
}

fn drain_audio_feedback(dispatcher: &mut LocalDispatcher, audio: &mut AudioHandle) {
    for feedback in audio.drain_feedback() {
        let result = dispatcher.dispatch_domain(&DomainAction::AudioFeedback(feedback), audio);
        if !result.audio_effects.is_empty() {
            audio.apply_effects(
                dispatcher.state(),
                &result.audio_effects,
                result.needs_full_sync,
            );
        }
    }
}
