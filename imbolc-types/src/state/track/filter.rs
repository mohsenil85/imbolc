use serde::{Deserialize, Serialize};

use super::ModulatedParam;
use crate::{Param, ParamValue};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FilterType {
    Lpf,
    Hpf,
    Bpf,
    Notch,
    Comb,
    Allpass,
    Vowel,
    ResDrive,
    ParametricEq,
}

impl FilterType {
    pub fn name(&self) -> &'static str {
        match self {
            FilterType::Lpf => "Low-Pass",
            FilterType::Hpf => "High-Pass",
            FilterType::Bpf => "Band-Pass",
            FilterType::Notch => "Notch",
            FilterType::Comb => "Comb",
            FilterType::Allpass => "Allpass",
            FilterType::Vowel => "Vowel",
            FilterType::ResDrive => "ResDrive",
            FilterType::ParametricEq => "Parametric",
        }
    }

    /// Reverse lookup from name() or short variant.
    pub fn from_name(s: &str) -> Option<FilterType> {
        match s {
            "lpf" | "Low-Pass" => Some(FilterType::Lpf),
            "hpf" | "High-Pass" => Some(FilterType::Hpf),
            "bpf" | "Band-Pass" => Some(FilterType::Bpf),
            "notch" | "Notch" => Some(FilterType::Notch),
            "comb" | "Comb" => Some(FilterType::Comb),
            "allpass" | "Allpass" => Some(FilterType::Allpass),
            "vowel" | "Vowel" => Some(FilterType::Vowel),
            "resdrive" | "ResDrive" => Some(FilterType::ResDrive),
            "parametric" | "Parametric" | "ParametricEq" => Some(FilterType::ParametricEq),
            _ => None,
        }
    }

    /// Display labels for the cutoff and resonance parameters.
    pub fn param_labels(&self) -> (&'static str, &'static str) {
        match self {
            FilterType::ParametricEq => ("Frequency", "Q"),
            _ => ("Cutoff", "Resonance"),
        }
    }

    pub fn synth_def_name(&self) -> &'static str {
        match self {
            FilterType::Lpf => "imbolc_lpf",
            FilterType::Hpf => "imbolc_hpf",
            FilterType::Bpf => "imbolc_bpf",
            FilterType::Notch => "imbolc_notch",
            FilterType::Comb => "imbolc_comb",
            FilterType::Allpass => "imbolc_allpass",
            FilterType::Vowel => "imbolc_vowel",
            FilterType::ResDrive => "imbolc_resdrive",
            FilterType::ParametricEq => "imbolc_parametric_eq",
        }
    }

    pub fn synth_def_name_mono(&self) -> &'static str {
        match self {
            FilterType::Lpf => "imbolc_lpf_mono",
            FilterType::Hpf => "imbolc_hpf_mono",
            FilterType::Bpf => "imbolc_bpf_mono",
            FilterType::Notch => "imbolc_notch_mono",
            FilterType::Comb => "imbolc_comb_mono",
            FilterType::Allpass => "imbolc_allpass_mono",
            FilterType::Vowel => "imbolc_vowel_mono",
            FilterType::ResDrive => "imbolc_resdrive_mono",
            FilterType::ParametricEq => "imbolc_parametric_eq_mono",
        }
    }

    #[allow(dead_code)]
    pub fn all() -> Vec<FilterType> {
        vec![
            FilterType::Lpf,
            FilterType::Hpf,
            FilterType::Bpf,
            FilterType::Notch,
            FilterType::Comb,
            FilterType::Allpass,
            FilterType::Vowel,
            FilterType::ResDrive,
            FilterType::ParametricEq,
        ]
    }

    pub fn default_extra_params(&self) -> Vec<Param> {
        match self {
            FilterType::Vowel => vec![Param {
                name: "shape".to_string(),
                value: ParamValue::Float(0.0),
                min: 0.0,
                max: 1.0,
            }],
            FilterType::ResDrive => vec![Param {
                name: "drive".to_string(),
                value: ParamValue::Float(1.0),
                min: 1.0,
                max: 8.0,
            }],
            FilterType::ParametricEq => vec![Param {
                name: "gain".to_string(),
                value: ParamValue::Float(0.0),
                min: -24.0,
                max: 24.0,
            }],
            _ => vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub filter_type: FilterType,
    pub cutoff: ModulatedParam,
    pub resonance: ModulatedParam,
    pub extra_params: Vec<Param>,
    /// Whether the filter is enabled (bypassed when false).
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl FilterConfig {
    pub fn new(filter_type: FilterType) -> Self {
        Self {
            extra_params: filter_type.default_extra_params(),
            filter_type,
            cutoff: ModulatedParam {
                value: 1000.0,
                min: 20.0,
                max: 20000.0,
                mod_source: None,
            },
            resonance: ModulatedParam {
                value: 0.5,
                min: 0.0,
                max: 1.0,
                mod_source: None,
            },
            enabled: true,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EqBandType {
    HighPass,
    LowShelf,
    Bell,
    HighShelf,
    LowPass,
}

impl EqBandType {
    pub fn name(&self) -> &'static str {
        match self {
            EqBandType::HighPass => "HP",
            EqBandType::LowShelf => "LS",
            EqBandType::Bell => "BL",
            EqBandType::HighShelf => "HS",
            EqBandType::LowPass => "LP",
        }
    }

    /// Whether this band type has a gain parameter.
    pub fn has_gain(&self) -> bool {
        matches!(
            self,
            EqBandType::LowShelf | EqBandType::Bell | EqBandType::HighShelf
        )
    }

    /// Whether this band type has a Q parameter.
    pub fn has_q(&self) -> bool {
        matches!(
            self,
            EqBandType::LowShelf | EqBandType::Bell | EqBandType::HighShelf
        )
    }

    /// Whether this band type has a slope parameter (HPF/LPF only).
    pub fn has_slope(&self) -> bool {
        matches!(self, EqBandType::HighPass | EqBandType::LowPass)
    }

    /// Returns the allowed band types for a given band index.
    /// Bands 0 and 5 are fixed (HPF/LPF), bands 1-4 can switch.
    pub fn allowed_types(band_idx: usize) -> &'static [EqBandType] {
        match band_idx {
            0 => &[EqBandType::HighPass],
            5 => &[EqBandType::LowPass],
            _ => &[
                EqBandType::Bell,
                EqBandType::LowShelf,
                EqBandType::HighShelf,
            ],
        }
    }

    /// SuperCollider type index: HP=0, LS=1, Bell=2, HS=3, LP=4
    pub fn sc_type_index(&self) -> i32 {
        match self {
            EqBandType::HighPass => 0,
            EqBandType::LowShelf => 1,
            EqBandType::Bell => 2,
            EqBandType::HighShelf => 3,
            EqBandType::LowPass => 4,
        }
    }
}

/// Filter slope for HPF/LPF bands (number of cascaded biquad stages).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum FilterSlope {
    #[default]
    Db12,
    Db24,
    Db48,
}

impl FilterSlope {
    pub fn name(&self) -> &'static str {
        match self {
            FilterSlope::Db12 => "12",
            FilterSlope::Db24 => "24",
            FilterSlope::Db48 => "48",
        }
    }

    /// SuperCollider slope value: Db12=1, Db24=2, Db48=3
    pub fn sc_slope_value(&self) -> i32 {
        match self {
            FilterSlope::Db12 => 1,
            FilterSlope::Db24 => 2,
            FilterSlope::Db48 => 3,
        }
    }

    /// Cycle to the next slope value.
    pub fn next(&self) -> FilterSlope {
        match self {
            FilterSlope::Db12 => FilterSlope::Db24,
            FilterSlope::Db24 => FilterSlope::Db48,
            FilterSlope::Db48 => FilterSlope::Db12,
        }
    }

    /// Cycle to the previous slope value.
    pub fn prev(&self) -> FilterSlope {
        match self {
            FilterSlope::Db12 => FilterSlope::Db48,
            FilterSlope::Db24 => FilterSlope::Db12,
            FilterSlope::Db48 => FilterSlope::Db24,
        }
    }

    /// Convert from normalized 0.0-1.0: Db12=0.0, Db24=0.5, Db48=1.0
    pub fn from_normalized(v: f32) -> FilterSlope {
        if v < 0.33 {
            FilterSlope::Db12
        } else if v < 0.67 {
            FilterSlope::Db24
        } else {
            FilterSlope::Db48
        }
    }

    /// Convert to normalized 0.0-1.0
    pub fn to_normalized(&self) -> f32 {
        match self {
            FilterSlope::Db12 => 0.0,
            FilterSlope::Db24 => 0.5,
            FilterSlope::Db48 => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqBand {
    pub band_type: EqBandType,
    pub freq: f32,
    pub gain: f32,
    pub q: f32,
    pub enabled: bool,
    #[serde(default)]
    pub slope: FilterSlope,
}

pub const EQ_BAND_COUNT: usize = 6;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqConfig {
    pub bands: Vec<EqBand>,
    pub enabled: bool,
}

impl EqConfig {
    /// Validate and fix band count on deserialization.
    /// If Vec has wrong size, replace with default.
    pub fn validated(mut self) -> Self {
        if self.bands.len() != EQ_BAND_COUNT {
            self.bands = Self::default().bands;
        }
        self
    }
}

impl Default for EqConfig {
    fn default() -> Self {
        Self {
            bands: vec![
                EqBand {
                    band_type: EqBandType::HighPass,
                    freq: 20.0,
                    gain: 0.0,
                    q: 0.7,
                    enabled: true,
                    slope: FilterSlope::Db12,
                },
                EqBand {
                    band_type: EqBandType::LowShelf,
                    freq: 100.0,
                    gain: 0.0,
                    q: 0.7,
                    enabled: true,
                    slope: FilterSlope::Db12,
                },
                EqBand {
                    band_type: EqBandType::Bell,
                    freq: 500.0,
                    gain: 0.0,
                    q: 1.0,
                    enabled: true,
                    slope: FilterSlope::Db12,
                },
                EqBand {
                    band_type: EqBandType::Bell,
                    freq: 2500.0,
                    gain: 0.0,
                    q: 1.0,
                    enabled: true,
                    slope: FilterSlope::Db12,
                },
                EqBand {
                    band_type: EqBandType::HighShelf,
                    freq: 8000.0,
                    gain: 0.0,
                    q: 0.7,
                    enabled: true,
                    slope: FilterSlope::Db12,
                },
                EqBand {
                    band_type: EqBandType::LowPass,
                    freq: 20000.0,
                    gain: 0.0,
                    q: 0.7,
                    enabled: true,
                    slope: FilterSlope::Db12,
                },
            ],
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_type_name_non_empty() {
        for ft in FilterType::all() {
            assert!(!ft.name().is_empty(), "{:?} has empty name", ft);
        }
    }

    #[test]
    fn filter_type_synth_def_name_prefixed() {
        for ft in FilterType::all() {
            assert!(ft.synth_def_name().starts_with("imbolc_"), "{:?}", ft);
        }
    }

    #[test]
    fn filter_type_synth_def_name_mono_suffixed() {
        for ft in FilterType::all() {
            assert!(ft.synth_def_name_mono().ends_with("_mono"), "{:?}", ft);
        }
    }

    #[test]
    fn filter_type_all_returns_all_variants() {
        let all = FilterType::all();
        assert_eq!(all.len(), 9);
        // No duplicates
        let mut deduped = all.clone();
        deduped.dedup();
        assert_eq!(deduped.len(), 9);
    }

    #[test]
    fn filter_type_default_extra_params_vowel() {
        let params = FilterType::Vowel.default_extra_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "shape");
    }

    #[test]
    fn filter_type_default_extra_params_resdrive() {
        let params = FilterType::ResDrive.default_extra_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "drive");
    }

    #[test]
    fn filter_type_default_extra_params_parametric_eq() {
        let params = FilterType::ParametricEq.default_extra_params();
        assert_eq!(params.len(), 1);
        assert_eq!(params[0].name, "gain");
    }

    #[test]
    fn filter_type_default_extra_params_others_empty() {
        for ft in [
            FilterType::Lpf,
            FilterType::Hpf,
            FilterType::Bpf,
            FilterType::Notch,
            FilterType::Comb,
            FilterType::Allpass,
        ] {
            assert!(
                ft.default_extra_params().is_empty(),
                "{:?} should have no extra params",
                ft
            );
        }
    }

    #[test]
    fn filter_config_new_defaults() {
        let cfg = FilterConfig::new(FilterType::Lpf);
        assert_eq!(cfg.cutoff.value, 1000.0);
        assert_eq!(cfg.resonance.value, 0.5);
        assert!(cfg.enabled);
        assert_eq!(cfg.filter_type, FilterType::Lpf);
    }

    #[test]
    fn filter_config_new_vowel_has_extra_params() {
        let cfg = FilterConfig::new(FilterType::Vowel);
        assert_eq!(cfg.extra_params.len(), 1);
        assert_eq!(cfg.extra_params[0].name, "shape");
    }

    #[test]
    fn eq_config_default() {
        let eq = EqConfig::default();
        assert_eq!(eq.bands.len(), EQ_BAND_COUNT);
        assert_eq!(eq.bands[0].band_type, EqBandType::HighPass);
        assert_eq!(eq.bands[1].band_type, EqBandType::LowShelf);
        assert_eq!(eq.bands[2].band_type, EqBandType::Bell);
        assert_eq!(eq.bands[3].band_type, EqBandType::Bell);
        assert_eq!(eq.bands[4].band_type, EqBandType::HighShelf);
        assert_eq!(eq.bands[5].band_type, EqBandType::LowPass);
        assert!(eq.enabled);
    }

    #[test]
    fn eq_band_type_name() {
        assert_eq!(EqBandType::HighPass.name(), "HP");
        assert_eq!(EqBandType::LowShelf.name(), "LS");
        assert_eq!(EqBandType::Bell.name(), "BL");
        assert_eq!(EqBandType::HighShelf.name(), "HS");
        assert_eq!(EqBandType::LowPass.name(), "LP");
    }

    #[test]
    fn eq_band_type_helpers() {
        assert!(!EqBandType::HighPass.has_gain());
        assert!(EqBandType::Bell.has_gain());
        assert!(EqBandType::HighPass.has_slope());
        assert!(!EqBandType::Bell.has_slope());
    }

    #[test]
    fn filter_slope_cycle() {
        assert_eq!(FilterSlope::Db12.next(), FilterSlope::Db24);
        assert_eq!(FilterSlope::Db24.next(), FilterSlope::Db48);
        assert_eq!(FilterSlope::Db48.next(), FilterSlope::Db12);
    }

    #[test]
    fn eq_config_validated_wrong_size() {
        let mut eq = EqConfig::default();
        eq.bands.pop(); // 5 bands - wrong
        let eq = eq.validated();
        assert_eq!(eq.bands.len(), EQ_BAND_COUNT);
    }
}
