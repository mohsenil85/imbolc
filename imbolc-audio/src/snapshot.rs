use imbolc_types::AutomationLane;
use imbolc_types::PianoRollState;
use imbolc_types::{SessionState, TrackState};

pub type TrackSnapshot = TrackState;
pub type SessionSnapshot = SessionState;
pub type PianoRollSnapshot = PianoRollState;
pub type AutomationSnapshot = Vec<AutomationLane>;
