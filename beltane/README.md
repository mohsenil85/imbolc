# Beltane

Beltane is the modular rewrite phase of Imbolc.

This workspace scaffolds a strict hexagonal architecture where the core music logic is isolated from GUI, audio backend, and storage.

## Crates

- `beltane-domain`: pure domain state and invariants.
- `beltane-ports`: command/event/effect contracts and port traits.
- `beltane-application`: use-case orchestration (`command -> events + effects + view`).
- `beltane-runtime`: wiring layer that dispatches effects to adapters.
- `beltane-gui`: GUI adapter boundary (currently a thin action mapper stub).

## First Vertical Slice

Implemented slice: `TogglePlay`.

Flow:

1. GUI adapter maps user input to `Command::TogglePlay`.
2. Application toggles transport state in the domain.
3. Application emits `AppEvent::TransportToggled`.
4. Runtime applies `Effect::SyncAudioTransport` to the audio port.
5. Runtime publishes `TransportView` to projection/UI port.

This is intentionally small and compile-verified so future modules can follow the same pattern.
