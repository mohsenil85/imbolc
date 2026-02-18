//! Action types for the dispatch system.
//!
//! Most action types are re-exported from imbolc-types. This module defines
//! IoFeedback which references complex state types that stay in imbolc-core.

use std::path::PathBuf;

use crate::state::custom_synthdef::CustomSynthDef;
use crate::state::session::SessionState;
use crate::state::track_state::TrackState;

// Re-export all action types from imbolc-types
pub use imbolc_types::{
    Action, ArrangementAction, AudioEffect, AudioFeedback, AutomationAction, BusAction,
    ClickAction, DispatchResult, DomainAction, EqualizerParamKind, FileSelectAction,
    FilterParamKind, GenerativeAction, GroupAction, LfoParamKind, MasterAction, MidiAction,
    MixerAction, NavAction, NavIntent, PaneId, PianoRollAction, RoutedAction, SampleSlicerAction,
    SequencerAction, ServerAction, SessionAction, StatusEvent, ToggleResult, TrackAction,
    TrackUpdate, TunerAction, UiAction, VstParamAction, VstTarget,
};

/// Feedback from async I/O operations to the main thread.
/// This type stays in imbolc-core because it references SessionState and TrackState.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum IoFeedback {
    SaveComplete {
        id: u64,
        path: PathBuf,
        result: Result<String, String>,
    },
    AutosaveComplete {
        id: u64,
        path: PathBuf,
        result: Result<(), String>,
    },
    LoadComplete {
        id: u64,
        path: PathBuf,
        result: Result<(SessionState, TrackState, String), String>,
    },
    ImportSynthDefComplete {
        id: u64,
        result: Result<(CustomSynthDef, String, PathBuf), String>,
    },
    ImportSynthDefLoaded {
        id: u64,
        result: Result<String, String>,
    },
}
