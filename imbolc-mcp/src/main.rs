//! Imbolc MCP server binary.
//!
//! Runs as a stdio MCP server. Optionally connects to a running DAW
//! via Unix socket for live state queries and mutations.
//! Reference tools (list_instruments, list_effects, etc.) work standalone.

use std::sync::Arc;

use rmcp::transport::io::stdio;
use rmcp::ServiceExt;

use imbolc_mcp::ipc_client::IpcClient;
use imbolc_mcp::server::ImbolcServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Log to stderr (stdout is reserved for MCP protocol)
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    // Try to connect to the DAW; reference tools work without it
    let ipc = match IpcClient::connect() {
        Ok(client) => {
            if let Err(e) = client.hello("Claude MCP") {
                tracing::warn!("IPC hello failed: {}", e);
            }
            tracing::info!("Connected to Imbolc DAW");
            Some(Arc::new(client))
        }
        Err(e) => {
            tracing::info!("DAW not running ({}), reference tools only", e);
            None
        }
    };

    let service = ImbolcServer::new(ipc);
    let server = service.serve(stdio()).await.inspect_err(|e| {
        tracing::error!("MCP serve error: {:?}", e);
    })?;

    server.waiting().await?;
    Ok(())
}
