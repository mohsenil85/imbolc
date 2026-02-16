# Tasks

All remaining work, organized by priority tier.

See TASKS_DONE.md for completed work.

---

## Features

### LayerGroup Undo Support

**Sources:** Phase B7 (layer group EQ)

`LayerGroupAction` variants (ToggleEq, SetEqParam, and all effect
CRUD variants) are not undoable. They fall through to `_ => false` in
`is_undoable()` and `_ => UndoScope::Full` in `undo_scope()`. Adding
undo requires choosing appropriate scopes (Session for structural
toggles, skip for real-time param tweaks) and testing undo/redo
round-trips.

**Files:** `imbolc-core/src/state/undo.rs`, `imbolc-core/src/dispatch/bus.rs`

---

### MIDI Learn

"Wiggle a knob to assign it" workflow. CC mapping state exists but
there's no interactive UI for binding a physical controller to a
parameter. Should support learn mode where the next incoming CC
automatically maps to the selected target.

---

## Long-term

### Scaling Bottlenecks

See [plans/scaling-analysis.md](plans/scaling-analysis.md) for ranked analysis
of local and network scaling issues.

---

### Multi-track Audio Recording

**Sources:** NEXT_STEPS #8

Record live audio input to tracks. Requires `cpal` crate for audio
capture, waveform display, overdub sync.

---

### UI themes

**Sources:** R2 #19

All colors hardcoded in `imbolc-ui/src/ui/style.rs`. Define a `Theme` struct
with semantic color slots, ship 2-3 built-in themes (Default, Light,
High Contrast), store active theme in `AppState`, add theme
switcher. Large change touching every pane.

**Files:** `imbolc-ui/src/ui/style.rs`, `imbolc-types/src/state/mod.rs`, all panes

---

### VST Parameter Discovery

Replace synthetic 128-parameter placeholders with real parameter
names, units, and ranges from the plugin via SuperCollider OSC
replies. Currently usable but clunky — users see "Param 0", "Param 1"
instead of meaningful names.

---

### Sidechain Visualization

Compressor gain reduction meters, sidechain input indicators in the
mixer.

---

### Group/Bus Metering

Level meters for the 8 buses and master in the mixer view.

---

### Plugin Scanning/Cataloging

Automatic VST3 directory scanning instead of manual file
import. Plugin database with search, favorites, and categories.

---

### VST Preset/Program Browser

UI for browsing and loading VST presets and programs. Currently state
save/restore works but there's no preset management interface.

---

### Latency Compensation

Plugin delay compensation (PDC) for VST instruments and
effects. Report and compensate for processing latency to keep tracks
aligned.

---

### MIDI Clock Sync

Send and receive MIDI clock for synchronization with external hardware
and software. Tempo leader/follower modes.

---

### CPU/DSP Load Meter

Real-time display of SuperCollider CPU usage and DSP load. Warning
indicators when approaching capacity.
