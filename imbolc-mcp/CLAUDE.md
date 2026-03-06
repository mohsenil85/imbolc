# imbolc-mcp

MCP (Model Context Protocol) server for Imbolc DAW. Runs as a stdio MCP server process — one per Claude Code instance. Connects to a running DAW via Unix socket for live state queries and mutations.

## Architecture

```
Claude Code ──stdio──> imbolc-mcp ──Unix socket──> imbolc-ui (DAW)
```

Reference tools (list_instruments, list_effects, etc.) work standalone without a running DAW.

## Module Map

| File | Purpose |
|------|---------|
| `main.rs` | Binary entry: stdio MCP transport, optional IPC connect |
| `server.rs` | rmcp `ServerHandler`, tool registration via `#[tool_router]` |
| `ipc_client.rs` | Unix socket client, length-prefixed JSON framing |
| `state_view.rs` | Format DAW state concisely for Claude readability |
| `tools/reference.rs` | Standalone tools: list/describe instruments, effects, filters |

## MCP Tools

**Reference (standalone):** `list_instruments`, `list_effects`, `list_filters`, `describe_instrument`, `describe_effect`, `search_instruments`

**Read (need DAW):** `get_status`, `get_tracks`, `get_track`, `get_session`, `get_effects`

**Write (need DAW):** `dispatch_command` — executes any REPL command

## Dependencies

- `imbolc-types` with `ipc` feature (protocol types, metadata catalogs)
- `rmcp` (MCP server framework)
- `tokio` (async runtime for stdio transport)

## Running

Standalone (reference tools only):
```bash
cargo run -p imbolc-mcp
```

With DAW:
```bash
# Terminal 1: Start DAW with MCP listener
cargo run -p imbolc-ui --features mcp

# Terminal 2: MCP server connects automatically
cargo run -p imbolc-mcp
```

Socket path: `~/.config/imbolc/mcp.sock`

## Adding a New Tool

1. Add parameter struct if needed (e.g. `MyParams` with `schemars::JsonSchema`)
2. Add method to `ImbolcServer` in `server.rs` with `#[tool(description = "...")]`
3. For IPC tools, use `self.with_ipc(|ipc| { ... })` pattern
