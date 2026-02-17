use imbolc_types::AutomationLane;
use imbolc_types::PianoRollState;
use imbolc_types::{SessionState, TrackState};

pub type InstrumentSnapshot = TrackState;
pub type SessionSnapshot = SessionState;
pub type PianoRollSnapshot = PianoRollState;
pub type AutomationSnapshot = Vec<AutomationLane>;
