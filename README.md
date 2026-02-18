# Imbolc

Imbolc is a complete music studio that runs right in your terminal. Write beats, layer synthesizers, shape sounds with effects, mix tracks, and record finished songs -- all without leaving your keyboard. It's free, open-source, and yours to keep.

**Terminal requirement:** Imbolc requires a terminal that supports the [Kitty keyboard protocol](https://sw.kovidgoyal.net/kitty/keyboard-protocol/), such as [Kitty](https://sw.kovidgoyal.net/kitty/) or [Ghostty](https://ghostty.org/). The default macOS Terminal and GNOME Terminal are not supported.

## Highlights

- **Supercollider** -- uses Supercollider for DSP and sythdefs
- **58 built-in instruments** -- oscillators, FM synthesis, physical models, drums, classic synths, samplers, and more
- **39 built-in effects** -- delays, reverbs, compressors, modulation, distortion, granular, spectral, and more
- **Piano roll and drum sequencer** -- per-note velocity and probability, variable grid resolution, groove and humanize
- **Semi-modular audio engine** -- per-track signal chain: source → LFO → effects → 12-band parametric EQ → mixer with bus routing
- **Mixer with up to 32 buses** -- per-track levels, pan, mute/solo, sends, and master control
- **Real-time LAN collaboration** -- multiple players on a shared session over your local network
- **Automation, arpeggiator, and recording** -- automate any parameter, arpeggiate chords, record and export to WAV

## Quick Start

1. Install [Rust](https://rustup.rs/) and [SuperCollider](https://supercollider.github.io/) (`scsynth` must be on your PATH).
2. Compile SynthDefs: `imbolc-core/bin/compile-synthdefs`
3. Run: `cargo run -p imbolc-ui --release`

## Press the buttons

- F1 - Create instruments and groups
- F2 - Edit patterns
- F3 - Track View
- F4 - Mixer
- F12 - Server settings
- ? - Show keybindings
- : - Invoke
- ; - Pane Switcher
- ^P - Repl
- ^M - Midi settings
- ^Q - Quit

## Documentation

- [Documentation](docs/README.md)
- [Installation Guide](docs/how-to/installation.md)
- [First Session Tutorial](docs/tutorials/first-session.md)

## License

This project is licensed under the GNU GPL v3.0. See [LICENSE](LICENSE) for details.

## Support the Project

Imbolc is free and open source. If you find it useful, consider supporting development:

- [GitHub Sponsors](https://github.com/sponsors/mohsenil85)
- [Ko-fi](https://ko-fi.com/mohsenil85)
