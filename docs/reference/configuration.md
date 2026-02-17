# Configuration Reference

All configuration files, paths, and environment variables for Imbolc.

---

## Config Files

| File | Purpose |
|------|---------|
| `~/.config/imbolc/config.toml` | Musical defaults and runtime settings |
| `~/.config/imbolc/keybindings.toml` | Keybinding overrides |
| `~/.config/imbolc/audio_devices.json` | Audio device preferences |

All config files are optional. Missing files or fields fall back to built-in defaults.

---

## Musical Defaults (`config.toml`)

```toml
[defaults]
bpm = 120
key = "C"
scale = "Major"
tuning_a4 = 432.0
time_signature = [4, 4]
snap = false
tuning = "EqualTemperament"
ji_flavor = "FiveLimit"
keyboard_layout = "qwerty"

[runtime]
autosave = true
autosave_interval_minutes = 2
```

### Defaults Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `bpm` | integer | `120` | Beats per minute |
| `key` | string | `"C"` | Musical key (`C`, `C#`, `D`, `D#`, `E`, `F`, `F#`, `G`, `G#`, `A`, `A#`, `B`) |
| `scale` | string | `"Major"` | Scale (`Major`, `Minor`, `Dorian`, `Phrygian`, `Lydian`, `Mixolydian`, `Aeolian`, `Locrian`, `Pentatonic`, `Blues`, `Chromatic`) |
| `tuning_a4` | float | `432.0` | Reference pitch for A4 in Hz |
| `time_signature` | array | `[4, 4]` | Time signature as `[beats, beat_value]` |
| `snap` | bool | `false` | Snap notes to grid by default |
| `tuning` | string | `"EqualTemperament"` | Tuning system (`EqualTemperament`, `ScaleJI`, `ChordJI`, `AdaptiveJI`, `GlobalJI`) |
| `ji_flavor` | string | `"FiveLimit"` | Just intonation flavor (`FiveLimit`, `SevenLimit`, `Pythagorean`) |
| `keyboard_layout` | string | `"qwerty"` | Keyboard layout for performance mode (`qwerty`, `colemak`) |
| `bus_count` | integer | 4 | Number of mixing buses for new projects |

### Runtime Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `autosave` | bool | `true` | Enable periodic crash-recovery snapshots |
| `autosave_interval_minutes` | integer | `2` | Interval between autosave snapshots (1-10080 minutes) |

---

## Project Files

| Path | Purpose |
|------|---------|
| `~/.config/imbolc/default.sqlite` | Default project database |
| `~/.config/imbolc/.imbolc.autosave` | Autosave crash-recovery snapshot |

Projects are SQLite databases (`.imbolc` or `.sqlite` extension). Use `Ctrl+S` (Save As) to choose a custom location. Use `Ctrl+o` to browse recent projects.

---

## Output Paths

| Output | Path |
|--------|------|
| Master recording | `master_<timestamp>.wav` in current working directory |
| Track render | `~/.config/imbolc/renders/render_<trackid>_<timestamp>.wav` |
| Master bounce | `~/.config/imbolc/exports/bounce_<timestamp>.wav` |
| Stem export | `~/.config/imbolc/exports/stem_<name>_<timestamp>.wav` |

Master recording is toggled with `Ctrl+r`. Track render, master bounce, and stem export are triggered from the Piano Roll pane (`R`, `B`, and `Ctrl+b` respectively).

---

## SynthDef Paths

| Path | Purpose |
|------|---------|
| Built-in synthdefs | Bundled with Imbolc (resolved automatically at compile time) |
| `~/.config/imbolc/synthdefs/` | User custom SynthDefs |

Place custom `.scsyndef` files in the user synthdefs directory. They will be available as Custom instrument types after loading synthdefs from the Audio Server pane (F5).

---

## VST

| Path | Purpose |
|------|---------|
| `~/.config/imbolc/vst_states/*.fxp` | Saved VST plugin state files |

VST plugin state is saved per-track and restored on project load.

---

## Log Files

| Path | Purpose |
|------|---------|
| `~/.config/imbolc/scsynth.log` | SuperCollider server log |
| `~/.config/imbolc/imbolc.log` | Application log |

---

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `IMBOLC_NO_AUDIO` | Set to `1` to run without audio (UI-only mode) |
| `IMBOLC_SYNTHDEFS_DIR` | Override the built-in SynthDef directory path |
