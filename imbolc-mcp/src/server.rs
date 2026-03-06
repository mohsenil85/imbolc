//! MCP server handler — registers tools and dispatches to IPC.

use std::sync::Arc;

use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router, ServerHandler,
};

use crate::ipc_client::IpcClient;
use crate::state_view;
use crate::tools::reference;
use imbolc_types::ipc::*;

/// The Imbolc MCP server.
#[derive(Clone)]
pub struct ImbolcServer {
    tool_router: ToolRouter<Self>,
    ipc: Option<Arc<IpcClient>>,
}

// ============================================================================
// Tool parameter types
// ============================================================================

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct DescribeParams {
    #[schemars(description = "Name of the instrument or effect (e.g. 'rhodes', 'saw', 'reverb')")]
    pub name: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct TrackIdParams {
    #[schemars(description = "Track ID (numeric)")]
    pub track_id: u32,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct CommandParams {
    #[schemars(
        description = "REPL command string (e.g. 'add saw', 'set bpm 140', 'note 60 0 480')"
    )]
    pub command: String,
}

#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct SearchParams {
    #[schemars(description = "Search term — tag, category, or keyword")]
    pub query: String,
}

// ============================================================================
// Tool implementations
// ============================================================================

#[tool_router]
impl ImbolcServer {
    pub fn new(ipc: Option<Arc<IpcClient>>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            ipc,
        }
    }

    // --- Reference tools (standalone) ---

    #[tool(
        description = "List all available instruments with categories, descriptions, tags, aliases, and use cases. Works without a running DAW."
    )]
    fn list_instruments(&self) -> String {
        reference::list_instruments()
    }

    #[tool(
        description = "List all available audio effects with categories, descriptions, tags, aliases, and use cases. Works without a running DAW."
    )]
    fn list_effects(&self) -> String {
        reference::list_effects()
    }

    #[tool(description = "List all available filter types. Works without a running DAW.")]
    fn list_filters(&self) -> String {
        reference::list_filters()
    }

    #[tool(
        description = "Describe a specific instrument in detail — parameters, ranges, use cases. Accepts natural names like 'rhodes', 'saw', '303'."
    )]
    fn describe_instrument(
        &self,
        Parameters(DescribeParams { name }): Parameters<DescribeParams>,
    ) -> String {
        reference::describe_instrument(&name)
    }

    #[tool(
        description = "Describe a specific effect in detail — parameters, ranges, use cases. Accepts natural names like 'reverb', 'echo', 'compressor'."
    )]
    fn describe_effect(
        &self,
        Parameters(DescribeParams { name }): Parameters<DescribeParams>,
    ) -> String {
        reference::describe_effect(&name)
    }

    #[tool(
        description = "Search instruments by tag or category (e.g. 'warm', 'percussive', 'Physical Models'). Works without a running DAW."
    )]
    fn search_instruments(
        &self,
        Parameters(SearchParams { query }): Parameters<SearchParams>,
    ) -> String {
        let by_tag = imbolc_types::instruments_by_tag(&query);
        let by_cat = imbolc_types::instruments_by_category(&query);

        let mut results: Vec<&imbolc_types::InstrumentMeta> = by_tag;
        for m in by_cat {
            if !results.iter().any(|r| r.source_type == m.source_type) {
                results.push(m);
            }
        }

        if results.is_empty() {
            format!("No instruments found matching '{}'. Try a tag like 'warm', 'percussive', or a category like 'Drums', 'Oscillators'.", query)
        } else {
            results
                .iter()
                .map(|m| format!("- **{}** (`{}`) — {}", m.name, m.short_name, m.description))
                .collect::<Vec<_>>()
                .join("\n")
        }
    }

    // --- Read tools (need DAW) ---

    #[tool(
        description = "Get DAW status: transport state, BPM, key, scale, track count, project path. Requires a running DAW."
    )]
    fn get_status(&self) -> String {
        self.with_ipc(|ipc| {
            let resp = ipc.request(&IpcRequest::GetStatus)?;
            match resp {
                IpcResponse::Status(info) => Ok(state_view::format_status(&info)),
                other => Ok(format!("Unexpected response: {:?}", other)),
            }
        })
    }

    #[tool(
        description = "Get all tracks with name, source type, level, pan, mute/solo. Requires a running DAW."
    )]
    fn get_tracks(&self) -> String {
        self.with_ipc(|ipc| {
            let resp = ipc.request(&IpcRequest::GetTracks)?;
            match resp {
                IpcResponse::Tracks(tracks) => Ok(state_view::format_tracks(&tracks)),
                other => Ok(format!("Unexpected response: {:?}", other)),
            }
        })
    }

    #[tool(
        description = "Get full detail for one track: params, effects, filter, LFO, envelope. Requires a running DAW."
    )]
    fn get_track(
        &self,
        Parameters(TrackIdParams { track_id }): Parameters<TrackIdParams>,
    ) -> String {
        self.with_ipc(|ipc| {
            let resp = ipc.request(&IpcRequest::GetTrackDetail {
                track_id: imbolc_types::TrackId::new(track_id),
            })?;
            match resp {
                IpcResponse::TrackDetail(track) => Ok(state_view::format_track_detail(&track)),
                IpcResponse::TrackNotFound(id) => Ok(format!("Track {} not found.", id)),
                other => Ok(format!("Unexpected response: {:?}", other)),
            }
        })
    }

    #[tool(
        description = "Get session state: BPM, key, scale, tuning, time signature. Requires a running DAW."
    )]
    fn get_session(&self) -> String {
        self.with_ipc(|ipc| {
            let resp = ipc.request(&IpcRequest::GetSession)?;
            match resp {
                IpcResponse::Session(session) => Ok(serde_json::to_string_pretty(&*session)
                    .unwrap_or_else(|_| "Failed to serialize session".to_string())),
                other => Ok(format!("Unexpected response: {:?}", other)),
            }
        })
    }

    #[tool(description = "Get effects chain for a specific track. Requires a running DAW.")]
    fn get_effects(
        &self,
        Parameters(TrackIdParams { track_id }): Parameters<TrackIdParams>,
    ) -> String {
        self.with_ipc(|ipc| {
            let resp = ipc.request(&IpcRequest::GetEffects {
                track_id: imbolc_types::TrackId::new(track_id),
            })?;
            match resp {
                IpcResponse::Effects(effects) => {
                    if effects.is_empty() {
                        Ok("No effects.".to_string())
                    } else {
                        Ok(effects
                            .iter()
                            .enumerate()
                            .map(|(i, e)| {
                                let params: Vec<String> = e
                                    .params
                                    .iter()
                                    .map(|p| format!("{}={}", p.name, p.value.to_f32()))
                                    .collect();
                                format!(
                                    "{}. {} {} — {}",
                                    i + 1,
                                    e.effect_type.name(),
                                    if e.enabled { "" } else { "(bypassed)" },
                                    params.join(", "),
                                )
                            })
                            .collect::<Vec<_>>()
                            .join("\n"))
                    }
                }
                IpcResponse::TrackNotFound(id) => Ok(format!("Track {} not found.", id)),
                other => Ok(format!("Unexpected response: {:?}", other)),
            }
        })
    }

    // --- Write tools (need DAW) ---

    #[tool(
        description = "Execute a REPL command in the DAW. Examples: 'add saw', 'set bpm 140', 'note 60 0 480 100', 'effect reverb 1', 'save'. Requires a running DAW."
    )]
    fn dispatch_command(
        &self,
        Parameters(CommandParams { command }): Parameters<CommandParams>,
    ) -> String {
        self.with_ipc(|ipc| {
            let resp = ipc.request(&IpcRequest::DispatchCommand(command))?;
            match resp {
                IpcResponse::CommandResult { output } => {
                    Ok(output.unwrap_or_else(|| "OK".to_string()))
                }
                IpcResponse::CommandError(e) => Ok(format!("Error: {}", e)),
                other => Ok(format!("Unexpected response: {:?}", other)),
            }
        })
    }
}

// ============================================================================
// ServerHandler
// ============================================================================

#[tool_handler]
impl ServerHandler for ImbolcServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Imbolc DAW MCP server. Use reference tools (list_instruments, list_effects, \
                 describe_instrument, describe_effect, search_instruments) to explore available \
                 sounds. Use read tools (get_status, get_tracks, get_track, get_session, \
                 get_effects) to inspect DAW state. Use dispatch_command to make changes \
                 (e.g. 'add saw', 'set bpm 140', 'note 60 0 480')."
                .to_string(),
        )
    }
}

// ============================================================================
// Helpers
// ============================================================================

impl ImbolcServer {
    /// Execute an IPC call, returning an error message if not connected.
    fn with_ipc<F>(&self, f: F) -> String
    where
        F: FnOnce(&IpcClient) -> std::io::Result<String>,
    {
        match &self.ipc {
            Some(ipc) => match f(ipc) {
                Ok(result) => result,
                Err(e) => format!("IPC error: {}. Is the DAW running with --features mcp?", e),
            },
            None => {
                "Not connected to DAW. Start Imbolc with: cargo run -p imbolc-ui --features mcp"
                    .to_string()
            }
        }
    }
}
