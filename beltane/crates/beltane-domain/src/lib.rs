//! Domain model for the Beltane rewrite workspace.
//! This crate must stay pure: no I/O, no GUI, no backend concerns.

/// Core transport state owned by the domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Transport {
    is_playing: bool,
}

impl Transport {
    /// Returns the current play state.
    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    /// Toggles play state and emits a domain event.
    pub fn toggle_play(&mut self) -> DomainEvent {
        self.is_playing = !self.is_playing;
        DomainEvent::TransportToggled {
            is_playing: self.is_playing,
        }
    }
}

/// Domain-level events emitted after successful state transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DomainEvent {
    TransportToggled { is_playing: bool },
}

/// Domain aggregate root for this initial scaffold.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Session {
    transport: Transport,
}

impl Session {
    /// Returns transport snapshot.
    pub fn transport(&self) -> Transport {
        self.transport
    }

    /// Domain use-case: toggle transport play state.
    pub fn toggle_transport_play(&mut self) -> DomainEvent {
        self.transport.toggle_play()
    }
}

#[cfg(test)]
mod tests {
    use super::{DomainEvent, Session};

    #[test]
    fn transport_toggle_emits_expected_events() {
        let mut session = Session::default();

        assert_eq!(
            session.toggle_transport_play(),
            DomainEvent::TransportToggled { is_playing: true }
        );
        assert_eq!(
            session.toggle_transport_play(),
            DomainEvent::TransportToggled { is_playing: false }
        );
    }
}
