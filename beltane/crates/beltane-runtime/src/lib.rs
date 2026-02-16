//! Runtime wiring for Beltane.
//! Applies application effects to concrete adapters via port traits.

use beltane_application::BeltaneApp;
use beltane_ports::{AppEvent, AudioPort, Command, Effect, ProjectionPort};

#[derive(Debug)]
pub struct Runtime<A, P>
where
    A: AudioPort,
    P: ProjectionPort,
{
    app: BeltaneApp,
    audio: A,
    projection: P,
}

impl<A, P> Runtime<A, P>
where
    A: AudioPort,
    P: ProjectionPort,
{
    pub fn new(app: BeltaneApp, audio: A, projection: P) -> Self {
        Self {
            app,
            audio,
            projection,
        }
    }

    pub fn with_defaults(audio: A, projection: P) -> Self {
        Self::new(BeltaneApp::default(), audio, projection)
    }

    /// Dispatches one command through the hex flow.
    pub fn dispatch(&mut self, command: Command) -> Vec<AppEvent> {
        let output = self.app.handle(command);

        for effect in output.effects {
            self.apply_effect(effect);
        }

        self.projection.publish_transport(output.view);
        output.events
    }

    pub fn app(&self) -> &BeltaneApp {
        &self.app
    }

    pub fn audio(&self) -> &A {
        &self.audio
    }

    pub fn projection(&self) -> &P {
        &self.projection
    }

    pub fn into_parts(self) -> (BeltaneApp, A, P) {
        (self.app, self.audio, self.projection)
    }

    fn apply_effect(&mut self, effect: Effect) {
        match effect {
            Effect::SyncAudioTransport { is_playing } => {
                self.audio.sync_transport(is_playing);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use beltane_ports::{AppEvent, AudioPort, Command, ProjectionPort, TransportView};

    use super::Runtime;

    #[derive(Debug, Default)]
    struct FakeAudio {
        synced_states: Vec<bool>,
    }

    impl AudioPort for FakeAudio {
        fn sync_transport(&mut self, is_playing: bool) {
            self.synced_states.push(is_playing);
        }
    }

    #[derive(Debug, Default)]
    struct FakeProjection {
        views: Vec<TransportView>,
    }

    impl ProjectionPort for FakeProjection {
        fn publish_transport(&mut self, view: TransportView) {
            self.views.push(view);
        }
    }

    #[test]
    fn runtime_routes_effects_and_projection() {
        let mut runtime = Runtime::with_defaults(FakeAudio::default(), FakeProjection::default());

        let first_events = runtime.dispatch(Command::TogglePlay);
        let second_events = runtime.dispatch(Command::TogglePlay);

        assert_eq!(
            first_events,
            vec![AppEvent::TransportToggled { is_playing: true }]
        );
        assert_eq!(
            second_events,
            vec![AppEvent::TransportToggled { is_playing: false }]
        );

        assert_eq!(runtime.audio().synced_states, vec![true, false]);
        assert_eq!(
            runtime.projection().views,
            vec![
                TransportView { is_playing: true },
                TransportView { is_playing: false }
            ]
        );
    }
}
