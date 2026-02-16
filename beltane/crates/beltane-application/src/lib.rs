//! Application layer for Beltane.
//! Coordinates domain transitions and emits events/effects for adapters.

use beltane_domain::{DomainEvent, Session};
use beltane_ports::{AppEvent, Command, Effect, HandleOutput, TransportView};

/// Beltane application service and state owner.
#[derive(Debug, Default)]
pub struct BeltaneApp {
    session: Session,
}

impl BeltaneApp {
    pub fn new(session: Session) -> Self {
        Self { session }
    }

    /// Returns current projection snapshot.
    pub fn view(&self) -> TransportView {
        TransportView {
            is_playing: self.session.transport().is_playing(),
        }
    }

    /// Handles a command and returns events, effects, and projection.
    pub fn handle(&mut self, command: Command) -> HandleOutput {
        match command {
            Command::TogglePlay => {
                let domain_event = self.session.toggle_transport_play();
                let app_event = map_event(domain_event);
                let is_playing = match domain_event {
                    DomainEvent::TransportToggled { is_playing } => is_playing,
                };

                HandleOutput {
                    events: vec![app_event],
                    effects: vec![Effect::SyncAudioTransport { is_playing }],
                    view: TransportView { is_playing },
                }
            }
        }
    }
}

fn map_event(event: DomainEvent) -> AppEvent {
    match event {
        DomainEvent::TransportToggled { is_playing } => AppEvent::TransportToggled { is_playing },
    }
}

#[cfg(test)]
mod tests {
    use beltane_ports::{AppEvent, Command, Effect, TransportView};

    use super::BeltaneApp;

    #[test]
    fn toggle_play_emits_event_effect_and_projection() {
        let mut app = BeltaneApp::default();

        let output = app.handle(Command::TogglePlay);

        assert_eq!(
            output.events,
            vec![AppEvent::TransportToggled { is_playing: true }]
        );
        assert_eq!(
            output.effects,
            vec![Effect::SyncAudioTransport { is_playing: true }]
        );
        assert_eq!(output.view, TransportView { is_playing: true });
    }
}
