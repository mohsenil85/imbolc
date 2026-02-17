# LAN Collaboration

Imbolc supports real-time collaboration over a local network. One machine acts as the audio server; others connect as clients and share control of the session.

## Requirements

Build Imbolc with networking support:

```bash
cargo build -p imbolc-ui --release --features net
```

For automatic discovery on the local network:

```bash
cargo build -p imbolc-ui --release --features mdns
```

## Starting a Server

1. Launch Imbolc with the `net` feature on the machine that will run audio.
2. Start the audio server normally (`F5` → `s` → `c`).
3. The network server starts automatically and listens for connections.

## Connecting as a Client

1. Launch Imbolc with the `net` feature on another machine.
2. Connect to the server's IP address.
3. With `mdns`, servers are discovered automatically.

## How It Works

- The server is authoritative: it runs the audio engine and applies all state changes.
- Clients send control actions only — no audio is streamed over the network.
- State changes are broadcast to all clients as incremental patches (~30 Hz).
- The privilege model controls which client can make changes at any time. Press `Ctrl+P` to request privilege.

## Limitations

- LAN only — not designed for internet use.
- No audio streaming (all participants hear audio from the server machine only).
- Single privilege holder at a time.
