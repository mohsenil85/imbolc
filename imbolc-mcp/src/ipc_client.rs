//! IPC client — connects to the running DAW via Unix socket.

use std::io;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;

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
        stream.set_read_timeout(Some(std::time::Duration::from_secs(30)))?;
        Ok(Self {
            stream: Mutex::new(stream),
        })
    }

    /// Send a request and wait for the response.
    pub fn request(&self, req: &IpcRequest) -> io::Result<IpcResponse> {
        let mut stream = self.stream.lock().unwrap();
        write_ipc_message(&mut *stream, req)?;
        read_ipc_message(&mut *stream)
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
}
