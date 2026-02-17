mod effect;
mod source_type;

// Re-export types from imbolc-types
pub use imbolc_types::{
    decode_effect_cursor_from_slice, effects_max_cursor, track_row_count, track_row_info,
    track_section_for_row, EnvConfig, EqBand, EqBandType, EqConfig, FilterConfig, FilterType,
    GrooveConfig, GroupMixer, LfoConfig, LfoShape, MixerBus, MixerSend, ModSource, ModulatedParam,
    OutputTarget, ParameterTarget, ProcessingStage, SendTapPoint, SourceExtra, SwingGrid, Track,
    TrackSection,
};

// Re-export from local modules (adds extension traits)
pub use effect::{EffectId, EffectSlot, EffectType, EffectTypeExt};
pub use source_type::{SourceType, SourceTypeExt};

// Re-export TrackId from imbolc-types (it's the type alias)
pub use imbolc_types::TrackId;
