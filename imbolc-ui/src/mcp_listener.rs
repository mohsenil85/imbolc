//! MCP IPC listener — accepts connections from imbolc-mcp processes.
//!
//! Architecture: background listener thread accepts Unix socket connections.
//! Each connection spawns a handler thread that reads IpcRequests and sends
//! them to the main loop via mpsc. The main loop processes them each frame
//! and sends IpcResponses back via per-client channels.

use std::io;
use std::net::Shutdown;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use imbolc_types::ipc::*;
use imbolc_types::{NavAction, SourceExtra};

use crate::action::AudioEffect;
use crate::dispatch::LocalDispatcher;

/// A pending IPC request from a connected MCP client.
pub struct McpMessage {
    pub client_id: u32,
    pub request: IpcRequest,
    pub response_tx: Sender<IpcResponse>,
}

/// Handle to the running MCP listener — held by the runtime.
pub struct McpListenerHandle {
    /// Receives requests from all connected MCP clients.
    pub rx: Receiver<McpMessage>,
    /// The socket path, for cleanup on drop.
    socket_path: std::path::PathBuf,
}

impl Drop for McpListenerHandle {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

/// Start the MCP IPC listener on a Unix socket.
///
/// Returns a handle for the main loop to drain messages from.
pub fn start_listener() -> io::Result<McpListenerHandle> {
    let socket_path = default_socket_path();

    // Clean up stale socket
    if socket_path.exists() {
        let _ = std::fs::remove_file(&socket_path);
    }
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let listener = UnixListener::bind(&socket_path)?;
    listener.set_nonblocking(false)?;

    let (tx, rx) = mpsc::channel::<McpMessage>();
    let next_client_id = Arc::new(Mutex::new(1u32));

    thread::spawn(move || {
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client_id = {
                        let mut id = next_client_id.lock().unwrap();
                        let current = *id;
                        *id = id.wrapping_add(1);
                        current
                    };
                    let tx = tx.clone();
                    thread::spawn(move || {
                        handle_connection(client_id, stream, tx);
                    });
                }
                Err(e) => {
                    log::warn!("MCP listener accept error: {}", e);
                }
            }
        }
    });

    log::info!("MCP listener started on {:?}", socket_path);

    Ok(McpListenerHandle { rx, socket_path })
}

/// Handle a single MCP client connection.
fn handle_connection(client_id: u32, mut stream: UnixStream, main_tx: Sender<McpMessage>) {
    // macOS: ensure blocking mode on accepted connection
    let _ = stream.set_nonblocking(false);
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));

    loop {
        let request: IpcRequest = match read_ipc_message(&mut stream) {
            Ok(req) => req,
            Err(e) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    log::info!("MCP client {} disconnected", client_id);
                } else {
                    log::warn!("MCP client {} read error: {}", client_id, e);
                }
                break;
            }
        };

        // Create a one-shot channel for the response
        let (resp_tx, resp_rx) = mpsc::channel::<IpcResponse>();

        // Send to main loop
        if main_tx
            .send(McpMessage {
                client_id,
                request,
                response_tx: resp_tx,
            })
            .is_err()
        {
            log::info!("MCP main loop gone, closing client {}", client_id);
            break;
        }

        // Wait for response from main loop
        match resp_rx.recv() {
            Ok(response) => {
                if let Err(e) = write_ipc_message(&mut stream, &response) {
                    log::warn!("MCP client {} write error: {}", client_id, e);
                    break;
                }
            }
            Err(_) => {
                log::info!("MCP response channel closed for client {}", client_id);
                break;
            }
        }
    }

    let _ = stream.shutdown(Shutdown::Both);
}

/// Process all pending MCP requests in the main loop.
///
/// Call this each frame (like `drain_io_feedback`).
/// Returns the last nav action requested (if any) for the runtime to process.
pub fn drain_mcp_requests(
    handle: &McpListenerHandle,
    dispatcher: &mut LocalDispatcher,
    audio: &mut crate::audio::AudioHandle,
    pending_effects: &mut Vec<AudioEffect>,
) -> Option<NavAction> {
    let mut nav = None;
    while let Ok(msg) = handle.rx.try_recv() {
        let (response, req_nav) = process_request(
            msg.client_id,
            msg.request,
            dispatcher,
            audio,
            pending_effects,
        );
        if req_nav.is_some() {
            nav = req_nav;
        }
        let _ = msg.response_tx.send(response);
    }
    nav
}

/// Collect status messages from a DispatchResult.
fn collect_status_messages(result: &imbolc_types::DispatchResult) -> Vec<String> {
    result.status.iter().map(|e| e.message.clone()).collect()
}

/// Process a single IPC request and produce a response.
/// Returns (response, optional nav action for the runtime to execute).
fn process_request(
    client_id: u32,
    request: IpcRequest,
    dispatcher: &mut LocalDispatcher,
    audio: &mut crate::audio::AudioHandle,
    pending_effects: &mut Vec<AudioEffect>,
) -> (IpcResponse, Option<NavAction>) {
    let mut nav = None;
    let response = match request {
        IpcRequest::Hello(info) => {
            log::info!("MCP client {} connected: {}", client_id, info.name);
            IpcResponse::Welcome { client_id }
        }
        IpcRequest::Ping => IpcResponse::Pong,

        // --- Queries ---
        IpcRequest::GetStatus => {
            let state = dispatcher.state();
            IpcResponse::Status(StatusInfo {
                bpm: state.session.bpm as f64,
                key: format!("{:?}", state.session.key),
                scale: format!("{:?}", state.session.scale),
                track_count: state.tracks.tracks.len(),
                playing: state.audio.playing,
                project_path: state.project.path.as_ref().map(|p| p.display().to_string()),
            })
        }
        IpcRequest::GetTracks => IpcResponse::Tracks(dispatcher.state().tracks.clone()),
        IpcRequest::GetTrackDetail { track_id } => {
            match dispatcher.state().tracks.track(track_id) {
                Some(track) => IpcResponse::TrackDetail(Box::new(track.clone())),
                None => IpcResponse::TrackNotFound(track_id),
            }
        }
        IpcRequest::GetSession => {
            IpcResponse::Session(Box::new(dispatcher.state().session.clone()))
        }
        IpcRequest::GetMixer => {
            IpcResponse::Mixer(Box::new(dispatcher.state().session.mixer.clone()))
        }
        IpcRequest::GetPianoRoll => {
            IpcResponse::PianoRoll(Box::new(dispatcher.state().session.piano_roll.clone()))
        }
        IpcRequest::GetSequencer { track_id } => match dispatcher.state().tracks.track(track_id) {
            Some(track) => match &track.source_extra {
                SourceExtra::Kit(seq) => IpcResponse::Sequencer(Box::new(seq.clone())),
                _ => IpcResponse::SequencerNotFound(track_id),
            },
            None => IpcResponse::SequencerNotFound(track_id),
        },
        IpcRequest::GetAutomation => {
            IpcResponse::Automation(Box::new(dispatcher.state().session.automation.clone()))
        }
        IpcRequest::GetArrangement => {
            IpcResponse::Arrangement(Box::new(dispatcher.state().session.arrangement.clone()))
        }
        IpcRequest::GetEffects { track_id } => match dispatcher.state().tracks.track(track_id) {
            Some(track) => {
                let effects: Vec<_> = track
                    .channel_strip
                    .processing_chain
                    .iter()
                    .filter_map(|stage| {
                        if let imbolc_types::ProcessingStage::Effect(slot) = stage {
                            Some(slot.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                IpcResponse::Effects(effects)
            }
            None => IpcResponse::TrackNotFound(track_id),
        },

        // --- Mutations ---
        IpcRequest::DispatchAction(action) => {
            let result = dispatcher.dispatch_domain(&action, audio);
            let msgs = collect_status_messages(&result);
            pending_effects.extend(result.audio_effects);

            IpcResponse::ActionOk {
                status_messages: msgs,
            }
        }
        IpcRequest::DispatchCommand(cmd) => {
            match crate::repl::parse_command(&cmd, dispatcher.state()) {
                Ok(cmd_result) => match cmd_result {
                    crate::repl::CommandResult::Action(domain) => {
                        let result = dispatcher.dispatch_domain(&domain, audio);
                        let msgs = collect_status_messages(&result);
                        pending_effects.extend(result.audio_effects);
                        IpcResponse::CommandResult {
                            output: if msgs.is_empty() {
                                None
                            } else {
                                Some(msgs.join("\n"))
                            },
                        }
                    }
                    crate::repl::CommandResult::Nav(nav_action) => {
                        let pane_name = match &nav_action {
                            NavAction::SwitchPane(id) => id.as_str(),
                            NavAction::PushPane(id) => id.as_str(),
                            NavAction::PopPane => "previous",
                        };
                        let output = format!("Switched to {}", pane_name);
                        nav = Some(nav_action);
                        IpcResponse::CommandResult {
                            output: Some(output),
                        }
                    }
                    crate::repl::CommandResult::Output(text) => {
                        IpcResponse::CommandResult { output: Some(text) }
                    }
                    crate::repl::CommandResult::Quit => {
                        IpcResponse::CommandError("Quit not supported via MCP".to_string())
                    }
                },
                Err(e) => IpcResponse::CommandError(e),
            }
        }
    };
    (response, nav)
}
