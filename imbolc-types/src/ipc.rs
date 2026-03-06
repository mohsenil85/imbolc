//! IPC protocol types for MCP ↔ DAW communication.
//!
//! Wire format: length-prefixed JSON over Unix socket.

use serde::{Deserialize, Serialize};

use crate::{
    ArrangementState, AutomationState, DomainAction, DrumSequencerState, EffectSlot, MixerState,
    PianoRollState, SessionState, Track, TrackId, TrackState,
};

/// MCP client identity — associated with a human collaborator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientInfo {
    pub name: String,
    pub owned_tracks: Vec<TrackId>,
    pub privileged: bool,
}

/// Request sent from an MCP process to the DAW.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcRequest {
    /// Handshake — identify this MCP client.
    Hello(McpClientInfo),
    /// Keepalive ping.
    Ping,

    // --- Queries ---
    /// Get transport status, BPM, track count.
    GetStatus,
    /// Get all tracks (summary).
    GetTracks,
    /// Get full detail for one track.
    GetTrackDetail { track_id: TrackId },
    /// Get session state (BPM, key, scale, etc.).
    GetSession,
    /// Get mixer state.
    GetMixer,
    /// Get piano roll state.
    GetPianoRoll,
    /// Get drum sequencer for a track.
    GetSequencer { track_id: TrackId },
    /// Get automation state.
    GetAutomation,
    /// Get arrangement state.
    GetArrangement,
    /// Get effects chain for a track.
    GetEffects { track_id: TrackId },

    // --- Mutations ---
    /// Dispatch a domain action.
    DispatchAction(DomainAction),
    /// Dispatch a raw REPL command string, parsed DAW-side.
    DispatchCommand(String),
}

/// Status summary sent in response to GetStatus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusInfo {
    pub bpm: f64,
    pub key: String,
    pub scale: String,
    pub track_count: usize,
    pub playing: bool,
    pub project_path: Option<String>,
}

/// Response sent from the DAW to an MCP process.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpcResponse {
    /// Handshake accepted.
    Welcome {
        client_id: u32,
    },
    /// Keepalive response.
    Pong,

    // --- Query responses ---
    Status(StatusInfo),
    Tracks(TrackState),
    TrackDetail(Box<Track>),
    TrackNotFound(TrackId),
    Session(Box<SessionState>),
    Mixer(Box<MixerState>),
    PianoRoll(Box<PianoRollState>),
    Sequencer(Box<DrumSequencerState>),
    SequencerNotFound(TrackId),
    Automation(Box<AutomationState>),
    Arrangement(Box<ArrangementState>),
    Effects(Vec<EffectSlot>),

    // --- Mutation responses ---
    ActionOk {
        status_messages: Vec<String>,
    },
    ActionRejected(String),
    ActionError(String),
    CommandResult {
        output: Option<String>,
    },
    CommandError(String),
}

// ============================================================================
// Length-prefixed JSON framing
// ============================================================================

/// Write a length-prefixed JSON message to a writer.
pub fn write_ipc_message<W: std::io::Write, T: Serialize>(
    writer: &mut W,
    msg: &T,
) -> std::io::Result<()> {
    let payload = serde_json::to_vec(msg)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let len = payload.len() as u32;
    writer.write_all(&len.to_be_bytes())?;
    writer.write_all(&payload)?;
    writer.flush()
}

/// Read a length-prefixed JSON message from a reader.
pub fn read_ipc_message<R: std::io::Read, T: serde::de::DeserializeOwned>(
    reader: &mut R,
) -> std::io::Result<T> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf)?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 100_000_000 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("IPC message too large: {} bytes", len),
        ));
    }

    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;

    serde_json::from_slice(&payload)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Default Unix socket path for MCP connections.
pub fn default_socket_path() -> std::path::PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("imbolc")
        .join("mcp.sock")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn roundtrip_ping_pong() {
        let mut buf = Vec::new();
        write_ipc_message(&mut buf, &IpcRequest::Ping).unwrap();

        let mut cursor = Cursor::new(&buf);
        let req: IpcRequest = read_ipc_message(&mut cursor).unwrap();
        assert!(matches!(req, IpcRequest::Ping));
    }

    #[test]
    fn roundtrip_hello() {
        let hello = IpcRequest::Hello(McpClientInfo {
            name: "Alice's Claude".to_string(),
            owned_tracks: vec![TrackId::new(1), TrackId::new(3)],
            privileged: false,
        });
        let mut buf = Vec::new();
        write_ipc_message(&mut buf, &hello).unwrap();

        let mut cursor = Cursor::new(&buf);
        let req: IpcRequest = read_ipc_message(&mut cursor).unwrap();
        match req {
            IpcRequest::Hello(info) => {
                assert_eq!(info.name, "Alice's Claude");
                assert_eq!(info.owned_tracks.len(), 2);
            }
            _ => panic!("expected Hello"),
        }
    }

    #[test]
    fn roundtrip_welcome() {
        let resp = IpcResponse::Welcome { client_id: 42 };
        let mut buf = Vec::new();
        write_ipc_message(&mut buf, &resp).unwrap();

        let mut cursor = Cursor::new(&buf);
        let r: IpcResponse = read_ipc_message(&mut cursor).unwrap();
        match r {
            IpcResponse::Welcome { client_id } => assert_eq!(client_id, 42),
            _ => panic!("expected Welcome"),
        }
    }

    #[test]
    fn roundtrip_status() {
        let resp = IpcResponse::Status(StatusInfo {
            bpm: 120.0,
            key: "C".to_string(),
            scale: "Major".to_string(),
            track_count: 4,
            playing: false,
            project_path: Some("/tmp/test.sqlite".to_string()),
        });
        let mut buf = Vec::new();
        write_ipc_message(&mut buf, &resp).unwrap();

        let mut cursor = Cursor::new(&buf);
        let r: IpcResponse = read_ipc_message(&mut cursor).unwrap();
        match r {
            IpcResponse::Status(info) => {
                assert_eq!(info.bpm, 120.0);
                assert_eq!(info.track_count, 4);
            }
            _ => panic!("expected Status"),
        }
    }

    #[test]
    fn roundtrip_action_ok() {
        let resp = IpcResponse::ActionOk {
            status_messages: vec!["Track added".to_string()],
        };
        let mut buf = Vec::new();
        write_ipc_message(&mut buf, &resp).unwrap();

        let mut cursor = Cursor::new(&buf);
        let r: IpcResponse = read_ipc_message(&mut cursor).unwrap();
        match r {
            IpcResponse::ActionOk { status_messages } => {
                assert_eq!(status_messages.len(), 1);
            }
            _ => panic!("expected ActionOk"),
        }
    }

    #[test]
    fn roundtrip_command() {
        let req = IpcRequest::DispatchCommand("add saw".to_string());
        let mut buf = Vec::new();
        write_ipc_message(&mut buf, &req).unwrap();

        let mut cursor = Cursor::new(&buf);
        let r: IpcRequest = read_ipc_message(&mut cursor).unwrap();
        match r {
            IpcRequest::DispatchCommand(cmd) => assert_eq!(cmd, "add saw"),
            _ => panic!("expected DispatchCommand"),
        }
    }
}
