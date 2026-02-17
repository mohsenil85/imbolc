# Using VST Plugins

Imbolc supports VST instruments and effects through SuperCollider's VSTPlugin extension. This is experimental.

## Prerequisites

1. Install the VSTPlugin extension for SuperCollider.
2. Compile VST wrapper SynthDefs:

```bash
imbolc-core/bin/compile-vst-synthdefs
```

3. In Imbolc, open the Server pane (`F5`) and load synthdefs (`l`).

## Adding a VST Instrument

1. Open the Instruments pane (`F1`).
2. Press `a` to add a track.
3. Select VST from the source type list.
4. In the Track Editor (`Enter`), load the VST plugin file (`.vst` or `.vst3`).

## Adding a VST Effect

1. Select a track and open the Track Editor (`F1` → `Enter`).
2. Press `a` to add an effect.
3. Select VST from the effect list.
4. Load the plugin file.

## Discovering Parameters

1. In the Track Editor, press `v` to open the VST Parameters pane.
2. Press `d` to discover/re-discover parameters from the plugin.
3. Use arrow keys to navigate parameters, `Left`/`Right` to adjust.
4. Press `r` to reset a parameter to its default.

## Automation

1. In the VST Parameters pane, select a parameter.
2. Press `a` to create an automation lane for it.
3. Switch to the Automation pane (`F7`) to draw automation curves.

## State Save/Restore

VST plugin state is saved automatically with your project as `.fxp` files in `~/.config/imbolc/vst_states/`. State restores automatically on project load.

## Current Limitations

- No plugin scanning or catalog browser — plugins are loaded manually by file path.
- No preset/program browser.
- No latency compensation (PDC).
- Limited MIDI-learn support in the UI.
