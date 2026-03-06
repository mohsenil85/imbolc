# imbolc-mcp

MCP server that lets Claude compose music through Imbolc. Runs as a stdio process — one per Claude Code session — and optionally connects to a running DAW for live control.

```
Claude Code ──stdio──> imbolc-mcp ──Unix socket──> imbolc-ui (DAW)
```

## Setup

Add to your project's `.mcp.json`:

```json
{
  "mcpServers": {
    "imbolc": {
      "command": "cargo",
      "args": ["run", "-p", "imbolc-mcp"]
    }
  }
}
```

Reference tools work immediately — no running DAW needed. For live control, start the DAW with the MCP listener:

```bash
cargo run -p imbolc-ui --features mcp
```

## Tools

### Reference (standalone)

Browse the instrument and effect catalog without a running DAW.

| Tool | Description |
|------|-------------|
| `list_instruments` | All instruments grouped by category — names, tags, aliases, use cases |
| `list_effects` | All effects grouped by category |
| `list_filters` | All filter types |
| `describe_instrument` | Detail for one instrument — parameters with ranges, use cases |
| `describe_effect` | Detail for one effect — parameters with ranges |
| `search_instruments` | Find instruments by tag (`"warm"`, `"percussive"`) or category (`"Drums"`) |

### Read (need DAW)

Query live DAW state.

| Tool | Description |
|------|-------------|
| `get_status` | Transport state, BPM, key, scale, track count, project path |
| `get_tracks` | All tracks — name, source type, level, pan, mute/solo |
| `get_track` | Full detail for one track — params, effects, filter, LFO, envelope |
| `get_session` | Session settings — BPM, key, scale, tuning, time signature |
| `get_effects` | Effects chain for a track with parameter values |

### Write (need DAW)

| Tool | Description |
|------|-------------|
| `dispatch_command` | Execute any REPL command in the DAW |

`dispatch_command` accepts the same commands as the built-in REPL. Examples:

```
track add saw              # Add a sawtooth track
track add-effect 0 reverb  # Add reverb to track 0
mixer adjust-level 0.1     # Bump selected track volume
piano-roll play-stop       # Toggle playback
session save               # Save project
```

## Multi-user

Multiple Claude instances can connect simultaneously. Each spawns its own `imbolc-mcp` process, all connecting to the same DAW socket at `~/.config/imbolc/mcp.sock`.

```
 Alice's Claude ──> imbolc-mcp ──┐
                                 ├──> imbolc-ui (single DAW)
 Bob's Claude   ──> imbolc-mcp ──┘
```

## Example session

```
You: "What instruments are available for bass?"

Claude uses search_instruments("bass") →
  - Saw (saw) — Sawtooth wave oscillator
  - Acid (acid) — TB-303 style acid bass
  - BassGuitar (bassg) — Electric bass guitar

You: "Add an acid bass and set the tempo to 130"

Claude uses dispatch_command("track add acid") → OK
Claude uses dispatch_command("session set-session 130") → OK

You: "What does the track look like?"

Claude uses get_tracks() →
  Track 1 "Acid-0" (Acid) — level: 0.80, pan: 0.0

Claude uses get_track(track_id: 0) →
  Track "Acid-0" (id: 0, source: Acid)
    Mixer: level=0.80, pan=0.0, mute=false, solo=false
    LFO: off | Envelope: A=0.010 D=0.10 S=0.70 R=0.30
    Params: cutoff=1000, resonance=0.7, ...
```

## Testing

```bash
cargo test -p imbolc-mcp
```

27 eval tests across three layers:

- **Reference evals** (13) — catalog correctness, natural language lookup
- **State view evals** (7) — formatting functions with known state
- **Scenario evals** (7) — full IPC pipeline via in-process MockDaw
