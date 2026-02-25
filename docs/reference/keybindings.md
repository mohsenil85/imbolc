# Keybindings Reference

Complete keyboard shortcut reference organized by context.

Press `?` in any pane for context-sensitive help. Keybindings can be customized -- see [Customizing Keybindings](../how-to/customizing-keybindings.md).

---

## Global

Available everywhere in Imbolc.

| Key | Description |
|-----|-------------|
| `Space` | Play / Stop |
| `Ctrl+r` | Toggle master recording |
| `F1` | Instruments pane |
| `F2` | Piano Roll / Sequencer / Waveform (context-driven) |
| `F3` | Track view |
| `F4` | Mixer |
| `F5` | Audio server |
| `F7` | Automation |
| `F8` | EQ (12-band) |
| `F9` | Groove settings |
| `F10` | Reference tuner |
| `F11` | Generative engine |
| `Ctrl+s` | Save |
| `Ctrl+S` | Save as |
| `Ctrl+l` | Load |
| `Ctrl+o` | Project browser |
| `Ctrl+z` | Undo |
| `Ctrl+y` | Redo |
| `Ctrl+c` | Copy |
| `Ctrl+x` | Cut |
| `Ctrl+v` | Paste |
| `Ctrl+a` | Select all |
| `Ctrl+n` | Add track |
| `Ctrl+d` | Delete track |
| `Ctrl+f` | Frame edit (BPM, key, scale, tuning) |
| `Ctrl+m` | MIDI settings |
| `Ctrl+t` | Parameter tags |
| `Ctrl+k` | Checkpoint list |
| `1`-`9`, `0` | Select track 1-10 |
| `_` | Two-digit track select |
| `<` / `>` | Previous / Next track |
| `/` | Toggle piano keyboard |
| `:` | Command palette |
| `;` | Pane switcher |
| `?` | Context help |
| `.` | Toggle master mute |
| `M` | Toggle metronome |
| `T` | Cycle UI theme |
| `` ` `` / `~` | Navigate back / forward |
| `Ctrl+q` | Quit |

---

## Instruments Pane

The main track list (F1). Create, select, and manage tracks.

| Key | Description |
|-----|-------------|
| `Up` / `Down` | Navigate tracks |
| `a` | Add track |
| `d` | Delete track |
| `Enter` | Edit track |
| `l` / `L` | Link / Unlink layer |
| `+` / `-` | Layer octave offset |
| `r` | Rename layer group |

---

## Track Editor

Edit source parameters, filter, effects, LFO, and envelope for the selected track. Open with `Enter` from the Instruments pane.

| Key | Description |
|-----|-------------|
| `Up` / `Down` | Navigate parameters |
| `Left` / `Right` | Adjust value |
| `PageUp` / `PageDown` | Coarse adjust (+/-10%) |
| `Ctrl+Left` / `Ctrl+Right` | Fine adjust |
| `Shift+Left` / `Shift+Right` | Musical/semantic adjust |
| `Tab` / `Shift+Tab` | Next / Previous section |
| `Enter` | Type value directly |
| `f` | Toggle filter |
| `t` | Cycle filter type |
| `e` | Toggle EQ |
| `l` | Toggle LFO |
| `s` | Cycle LFO shape |
| `m` | Cycle LFO target |
| `a` | Add effect |
| `d` | Remove effect |
| `b` | Toggle effect bypass |
| `p` | Toggle polyphonic |
| `C` | Toggle mono/stereo |
| `o` | Load sample |
| `v` | VST parameters |
| `\` | Zero parameter |
| `Delete` | Clear to minimum |
| `Shift+Delete` | Reset to default |
| `Ctrl+Up` / `Ctrl+Down` | Move processing stage |

---

## Piano Roll

Note editor with grid, selection, and zoom (F2 when a pitched instrument is selected).

| Key | Description |
|-----|-------------|
| Arrow keys | Move cursor |
| `Enter` | Place / Remove note |
| `+` / `-` | Velocity up / down |
| `Alt+Left` / `Alt+Right` | Shrink / Grow duration |
| `[` / `]` | Set loop start / end |
| `l` | Toggle loop |
| `PageUp` / `PageDown` | Scroll by octave |
| `Home` / `End` | Jump to start / end |
| `z` / `x` | Zoom in / out |
| `t` | Toggle note editor / step sequencer |
| `T` | Cycle time signature |
| `g` | Cycle step resolution |
| `m` | Toggle poly/mono |
| `Shift+Arrow` | Extend selection |
| `A` | Toggle automation overlay |
| `R` | Render track to WAV |
| `B` | Bounce master to WAV |
| `Ctrl+b` | Export stems |

---

## Drum Sequencer

Step sequencer for drum and Kit instruments (F2 when a Kit track is selected).

| Key | Description |
|-----|-------------|
| Arrow keys | Navigate pads / steps |
| `Enter` | Toggle step |
| `s` | Load sample for pad |
| `c` | Sample slicer |
| `x` / `X` | Clear pad / Clear pattern |
| `[` / `]` | Previous / Next pattern |
| `{` | Cycle pattern length |
| `g` | Cycle grid resolution |
| `+` / `_` | Step velocity up / down |
| `=` / `-` | Pad pitch up / down (semitone) |
| `r` | Toggle pad reverse |
| `i` / `I` | Assign / Clear instrument |
| `Ctrl+Left` / `Ctrl+Right` | Pad level |

---

## Mixer

Console view with faders, pan, sends, and bus routing (F4).

| Key | Description |
|-----|-------------|
| `Left` / `Right` | Select channel |
| `Up` / `Down` | Adjust level |
| `PageUp` / `PageDown` | Coarse level adjust |
| `m` | Toggle mute |
| `s` | Toggle solo |
| `p` / `P` | Pan left / right |
| `o` / `O` | Cycle output target |
| `Tab` | Cycle section |
| `Enter` | Enter channel detail |
| `a` | Add effect (detail) |
| `d` | Remove effect (detail) |
| `e` | Toggle effect bypass (detail) |
| `f` | Toggle filter (detail) |
| `t` / `T` | Next / Previous send target |
| `g` | Toggle send |

---

## Automation

Parameter automation editor (F7).

| Key | Description |
|-----|-------------|
| `Tab` | Switch focus (lane list / timeline) |
| Arrow keys | Navigate / Adjust |
| `a` | Add lane |
| `x` | Remove lane |
| `e` | Toggle lane enabled |
| `Enter` | Place / Remove point |
| `d` | Delete point |
| `c` | Cycle curve type |
| `C` | Clear lane |
| `r` | Toggle automation recording |
| `R` | Arm / Disarm lane |
| `z` / `Z` | Zoom in / out |

---

## EQ

12-band parametric equalizer (F8).

| Key | Description |
|-----|-------------|
| `Left` / `Right` | Select band |
| `Tab` / `Shift+Tab` | Next / Previous parameter |
| `Up` / `Down` | Adjust value |
| `e` | Toggle EQ on/off |
| `b` | Toggle band on/off |

---

## Track View

Timeline clip arrangement (F3).

| Key | Description |
|-----|-------------|
| `Up` / `Down` | Select lane |
| `Left` / `Right` | Move cursor |
| `n` / `N` | Capture clip / New empty clip |
| `p` | Place clip at cursor |
| `Enter` | Edit clip |
| `d` / `D` | Delete placement / Delete clip |
| `y` | Duplicate placement |
| `m` | Toggle Song/Pattern mode |
| `[` / `]` | Previous / Next clip |

---

## Audio Server

SuperCollider server control and diagnostics (F5).

| Key | Description |
|-----|-------------|
| `s` | Start scsynth |
| `k` | Kill scsynth |
| `c` | Connect |
| `d` | Disconnect |
| `b` | Build synthdefs |
| `v` | Build VST synthdefs |
| `l` | Load synthdefs |
| `r` | Refresh audio devices |

---

## Performance Mode (Piano)

Activated with `/`. Turns letter keys into a piano keyboard for live playing.

- `Escape` cycles keyboard layout (C / A) or exits performance mode
- `[` / `]` shift octave down / up
- All letter keys become piano keys (layout depends on C or A mode)

## Performance Mode (Pad)

4x4 pad grid for live drum input.

- `Escape` exits performance mode
