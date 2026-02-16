//! Port contracts for Beltane hexagonal architecture.

/// Inbound application commands from driving adapters (GUI, MIDI, network, etc).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    TogglePlay,
}

/// Application-level events exposed to observers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEvent {
    TransportToggled { is_playing: bool },
}

/// Outbound side effects requested by the application layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    SyncAudioTransport { is_playing: bool },
}

/// Projection used by UI render adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportView {
    pub is_playing: bool,
}

/// Standard response shape for command handling.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandleOutput {
    pub events: Vec<AppEvent>,
    pub effects: Vec<Effect>,
    pub view: TransportView,
}

/// Outbound port implemented by audio adapters.
pub trait AudioPort {
    fn sync_transport(&mut self, is_playing: bool);
}

/// Outbound port implemented by UI projection adapters.
pub trait ProjectionPort {
    fn publish_transport(&mut self, view: TransportView);
}
