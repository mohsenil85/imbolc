//! IPC client — connects to the running DAW via Unix socket.

use std::io;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use std::time::Duration;

use imbolc_types::ipc::*;

/// Connection to the running DAW instance.
pub struct IpcClient {
    stream: Mutex<UnixStream>,
}

impl IpcClient {
    /// Connect to the DAW's MCP socket.
    pub fn connect() -> io::Result<Self> {
        let path = default_socket_path();
        let stream = UnixStream::connect(&path)?;
        stream.set_read_timeout(Some(Duration::from_secs(30)))?;
        Ok(Self {
            stream: Mutex::new(stream),
        })
    }

    /// Send a request and wait for the response.
    /// On broken connection, attempts to reconnect and retry once.
    pub fn request(&self, req: &IpcRequest) -> io::Result<IpcResponse> {
        let mut stream = self.stream.lock().unwrap();

        match Self::try_request(&mut stream, req) {
            Ok(resp) => Ok(resp),
            Err(e) if Self::is_reconnectable(&e) => {
                tracing::info!("IPC connection lost, reconnecting...");
                let path = default_socket_path();
                let new_stream = UnixStream::connect(&path)?;
                new_stream.set_read_timeout(Some(Duration::from_secs(30)))?;
                *stream = new_stream;

                // Re-send hello handshake on new connection
                let hello = IpcRequest::Hello(McpClientInfo {
                    name: "Claude MCP".to_string(),
                    owned_tracks: vec![],
                    privileged: false,
                });
                write_ipc_message(&mut *stream, &hello)?;
                let _welcome: IpcResponse = read_ipc_message(&mut *stream)?;

                // Retry the original request
                Self::try_request(&mut stream, req)
            }
            Err(e) => Err(e),
        }
    }

    /// Send the Hello handshake.
    pub fn hello(&self, name: &str) -> io::Result<u32> {
        let resp = self.request(&IpcRequest::Hello(McpClientInfo {
            name: name.to_string(),
            owned_tracks: vec![],
            privileged: false,
        }))?;
        match resp {
            IpcResponse::Welcome { client_id } => Ok(client_id),
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unexpected response to Hello",
            )),
        }
    }

    fn try_request(stream: &mut UnixStream, req: &IpcRequest) -> io::Result<IpcResponse> {
        write_ipc_message(stream, req)?;
        read_ipc_message(stream)
    }

    fn is_reconnectable(e: &io::Error) -> bool {
        matches!(
            e.kind(),
            io::ErrorKind::BrokenPipe
                | io::ErrorKind::ConnectionReset
                | io::ErrorKind::UnexpectedEof
                | io::ErrorKind::NotConnected
        )
    }
}
