# Customizing Keybindings

Override any default keybinding by creating a TOML file at:

```
~/.config/imbolc/keybindings.toml
```

## Format

The file uses the same format as the built-in keybindings. You only need to include the bindings you want to change — everything else falls back to defaults.

```toml
version = 2

[layers.global]
bindings = [
  { key = "Ctrl+w", action = "quit", description = "Quit (custom)" },
]

[layers.piano_roll]
bindings = [
  { key = "n", action = "toggle_note", description = "Place note (custom)" },
]
```

## Finding Current Bindings

Press `?` in any pane to see all active keybindings for that context. The full default keybinding set is in `imbolc-ui/keybindings.toml` in the source repository.

## Available Layers

Each layer corresponds to a pane or mode: `global`, `instrument`, `mixer`, `piano_roll`, `sequencer`, `instrument_edit`, `server`, `automation`, `eq`, `groove`, `track`, `tuner`, `arpeggiator`, `generative`, `vst_params`, `waveform`, `midi_settings`, `tag_view`, and performance modes (`piano_mode`, `pad_mode`).

See the [Keybindings Reference](../reference/keybindings.md) for the complete default binding list.
