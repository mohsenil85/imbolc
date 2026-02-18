use serde::{Deserialize, Serialize};

/// Musical key (pitch class)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Key {
    C,
    Cs,
    D,
    Ds,
    E,
    F,
    Fs,
    G,
    Gs,
    A,
    As,
    B,
}

impl Key {
    pub const ALL: [Key; 12] = [
        Key::C,
        Key::Cs,
        Key::D,
        Key::Ds,
        Key::E,
        Key::F,
        Key::Fs,
        Key::G,
        Key::Gs,
        Key::A,
        Key::As,
        Key::B,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Key::C => "C",
            Key::Cs => "C#",
            Key::D => "D",
            Key::Ds => "D#",
            Key::E => "E",
            Key::F => "F",
            Key::Fs => "F#",
            Key::G => "G",
            Key::Gs => "G#",
            Key::A => "A",
            Key::As => "A#",
            Key::B => "B",
        }
    }

    pub fn from_name(s: &str) -> Option<Key> {
        match s {
            "C" => Some(Key::C),
            "C#" | "Cs" => Some(Key::Cs),
            "D" => Some(Key::D),
            "D#" | "Ds" => Some(Key::Ds),
            "E" => Some(Key::E),
            "F" => Some(Key::F),
            "F#" | "Fs" => Some(Key::Fs),
            "G" => Some(Key::G),
            "G#" | "Gs" => Some(Key::Gs),
            "A" => Some(Key::A),
            "A#" | "As" => Some(Key::As),
            "B" => Some(Key::B),
            _ => None,
        }
    }

    /// MIDI note number for this key in octave 0
    pub fn semitone(&self) -> i32 {
        match self {
            Key::C => 0,
            Key::Cs => 1,
            Key::D => 2,
            Key::Ds => 3,
            Key::E => 4,
            Key::F => 5,
            Key::Fs => 6,
            Key::G => 7,
            Key::Gs => 8,
            Key::A => 9,
            Key::As => 10,
            Key::B => 11,
        }
    }
}

/// Scale definition as intervals from root
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scale {
    Major,
    Minor,
    Dorian,
    Phrygian,
    Lydian,
    Mixolydian,
    Aeolian,
    Locrian,
    Pentatonic,
    Blues,
    Chromatic,
}

impl Scale {
    pub const ALL: [Scale; 11] = [
        Scale::Major,
        Scale::Minor,
        Scale::Dorian,
        Scale::Phrygian,
        Scale::Lydian,
        Scale::Mixolydian,
        Scale::Aeolian,
        Scale::Locrian,
        Scale::Pentatonic,
        Scale::Blues,
        Scale::Chromatic,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Scale::Major => "Major",
            Scale::Minor => "Minor",
            Scale::Dorian => "Dorian",
            Scale::Phrygian => "Phrygian",
            Scale::Lydian => "Lydian",
            Scale::Mixolydian => "Mixolydian",
            Scale::Aeolian => "Aeolian",
            Scale::Locrian => "Locrian",
            Scale::Pentatonic => "Pentatonic",
            Scale::Blues => "Blues",
            Scale::Chromatic => "Chromatic",
        }
    }

    pub fn from_name(s: &str) -> Option<Scale> {
        match s {
            "Major" | "major" => Some(Scale::Major),
            "Minor" | "minor" => Some(Scale::Minor),
            "Dorian" | "dorian" => Some(Scale::Dorian),
            "Phrygian" | "phrygian" => Some(Scale::Phrygian),
            "Lydian" | "lydian" => Some(Scale::Lydian),
            "Mixolydian" | "mixolydian" => Some(Scale::Mixolydian),
            "Aeolian" | "aeolian" => Some(Scale::Aeolian),
            "Locrian" | "locrian" => Some(Scale::Locrian),
            "Pentatonic" | "pentatonic" => Some(Scale::Pentatonic),
            "Blues" | "blues" => Some(Scale::Blues),
            "Chromatic" | "chromatic" => Some(Scale::Chromatic),
            _ => None,
        }
    }

    pub fn short_name(&self) -> &'static str {
        self.name()
    }

    /// Semitone intervals from root for this scale
    pub fn intervals(&self) -> &'static [i32] {
        match self {
            Scale::Major => &[0, 2, 4, 5, 7, 9, 11],
            Scale::Minor => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Dorian => &[0, 2, 3, 5, 7, 9, 10],
            Scale::Phrygian => &[0, 1, 3, 5, 7, 8, 10],
            Scale::Lydian => &[0, 2, 4, 6, 7, 9, 11],
            Scale::Mixolydian => &[0, 2, 4, 5, 7, 9, 10],
            Scale::Aeolian => &[0, 2, 3, 5, 7, 8, 10],
            Scale::Locrian => &[0, 1, 3, 5, 6, 8, 10],
            Scale::Pentatonic => &[0, 2, 4, 7, 9],
            Scale::Blues => &[0, 3, 5, 6, 7, 10],
            Scale::Chromatic => &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        }
    }
}

/// Tuning system for pitch-to-frequency conversion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Tuning {
    /// Standard 12-tone equal temperament
    #[default]
    EqualTemperament,
    /// Just intonation ratios relative to scale tonic
    ScaleJI,
    /// Just intonation ratios relative to detected chord root
    ChordJI,
    /// Chord JI with continuity constraints to minimize pitch drift
    AdaptiveJI,
    /// Chord JI with accumulated global drift (no snap-back)
    GlobalJI,
}

impl Tuning {
    pub const ALL: [Tuning; 5] = [
        Tuning::EqualTemperament,
        Tuning::ScaleJI,
        Tuning::ChordJI,
        Tuning::AdaptiveJI,
        Tuning::GlobalJI,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            Tuning::EqualTemperament => "Equal Temperament",
            Tuning::ScaleJI => "Scale JI",
            Tuning::ChordJI => "Chord JI",
            Tuning::AdaptiveJI => "Adaptive JI",
            Tuning::GlobalJI => "Global JI",
        }
    }

    pub fn from_name(s: &str) -> Option<Tuning> {
        match s {
            "EqualTemperament" | "ET" | "12-TET" => Some(Tuning::EqualTemperament),
            "ScaleJI" => Some(Tuning::ScaleJI),
            "ChordJI" => Some(Tuning::ChordJI),
            "AdaptiveJI" => Some(Tuning::AdaptiveJI),
            "GlobalJI" => Some(Tuning::GlobalJI),
            _ => None,
        }
    }

    pub fn short_name(&self) -> &'static str {
        match self {
            Tuning::EqualTemperament => "12-TET",
            Tuning::ScaleJI => "ScaleJI",
            Tuning::ChordJI => "ChrdJI",
            Tuning::AdaptiveJI => "AdptJI",
            Tuning::GlobalJI => "GlobJI",
        }
    }
}

/// JI ratio philosophy — controls which ratio table is used by all JI variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum JIFlavor {
    /// Classic 5-limit: 9/5 minor 7th
    #[default]
    FiveLimit,
    /// Septimal 7-limit: 7/4 minor 7th
    SevenLimit,
    /// Pure fifths: 16/9 minor 7th
    Pythagorean,
}

impl JIFlavor {
    pub const ALL: [JIFlavor; 3] = [
        JIFlavor::FiveLimit,
        JIFlavor::SevenLimit,
        JIFlavor::Pythagorean,
    ];

    pub fn name(&self) -> &'static str {
        match self {
            JIFlavor::FiveLimit => "5-Limit",
            JIFlavor::SevenLimit => "7-Limit",
            JIFlavor::Pythagorean => "Pythagorean",
        }
    }

    pub fn from_name(s: &str) -> Option<JIFlavor> {
        match s {
            "FiveLimit" | "5-Limit" | "5L" => Some(JIFlavor::FiveLimit),
            "SevenLimit" | "7-Limit" | "7L" => Some(JIFlavor::SevenLimit),
            "Pythagorean" | "pythagorean" => Some(JIFlavor::Pythagorean),
            _ => None,
        }
    }
}

/// For each pitch class 0–11, the semitone correction to snap to the nearest
/// scale degree in the given key+scale. E.g. C Major, pitch class 1 (C#)
/// → −1 (snap down to C) because C is closer than D.
pub fn scale_quantization_table(key: Key, scale: Scale) -> [f32; 12] {
    let root = key.semitone();
    let mut in_scale = [false; 12];
    for &interval in scale.intervals() {
        in_scale[((root + interval) % 12) as usize] = true;
    }

    let mut table = [0.0f32; 12];
    for pc in 0..12 {
        if in_scale[pc] {
            continue;
        }
        // Search outward for nearest in-scale pitch class.
        // On ties, prefer downward correction for deterministic behavior.
        for d in 1..=6 {
            let down = (pc as i32 - d).rem_euclid(12);
            if in_scale[down as usize] {
                table[pc] = -(d as f32);
                break;
            }
            let up = (pc as i32 + d) % 12;
            if in_scale[up as usize] {
                table[pc] = d as f32;
                break;
            }
        }
    }
    table
}

/// Semitone offset to compensate for tuning_a4 ≠ 440.
/// SC's `cpsmidi` assumes A4 = 440; this corrects detected MIDI values.
pub fn tuning_ref_offset(tuning_a4: f32) -> f32 {
    12.0 * (tuning_a4 / 440.0).log2()
}

/// Base key frequency in octave 4 for the current session key and tuning.
/// Example: A => A4, C => C4.
pub fn key_root_freq(key: Key, tuning_a4: f32) -> f32 {
    let midi = 60.0 + key.semitone() as f32;
    tuning_a4 * 2.0_f32.powf((midi - 69.0) / 12.0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn key_all_has_12() {
        assert_eq!(Key::ALL.len(), 12);
    }

    #[test]
    fn key_names_unique() {
        let names: HashSet<&str> = Key::ALL.iter().map(|k| k.name()).collect();
        assert_eq!(names.len(), 12);
    }

    #[test]
    fn key_semitones_0_to_11() {
        let semitones: Vec<i32> = Key::ALL.iter().map(|k| k.semitone()).collect();
        assert_eq!(semitones, (0..12).collect::<Vec<i32>>());
    }

    #[test]
    fn key_c_semitone_is_zero() {
        assert_eq!(Key::C.semitone(), 0);
    }

    #[test]
    fn scale_all_has_11() {
        assert_eq!(Scale::ALL.len(), 11);
    }

    #[test]
    fn scale_names_unique() {
        let names: HashSet<&str> = Scale::ALL.iter().map(|s| s.name()).collect();
        assert_eq!(names.len(), 11);
    }

    #[test]
    fn scale_major_intervals() {
        assert_eq!(Scale::Major.intervals(), &[0, 2, 4, 5, 7, 9, 11]);
    }

    #[test]
    fn scale_minor_intervals() {
        assert_eq!(Scale::Minor.intervals(), &[0, 2, 3, 5, 7, 8, 10]);
    }

    #[test]
    fn scale_chromatic_has_12_notes() {
        assert_eq!(Scale::Chromatic.intervals().len(), 12);
    }

    #[test]
    fn scale_pentatonic_has_5_notes() {
        assert_eq!(Scale::Pentatonic.intervals().len(), 5);
    }

    #[test]
    fn tuning_all_has_5() {
        assert_eq!(Tuning::ALL.len(), 5);
    }

    #[test]
    fn tuning_names_unique() {
        let names: HashSet<&str> = Tuning::ALL.iter().map(|t| t.name()).collect();
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn tuning_short_names_unique() {
        let names: HashSet<&str> = Tuning::ALL.iter().map(|t| t.short_name()).collect();
        assert_eq!(names.len(), 5);
    }

    #[test]
    fn tuning_default_is_et() {
        assert_eq!(Tuning::default(), Tuning::EqualTemperament);
    }

    #[test]
    fn ji_flavor_all_has_3() {
        assert_eq!(JIFlavor::ALL.len(), 3);
    }

    #[test]
    fn ji_flavor_names_unique() {
        let names: HashSet<&str> = JIFlavor::ALL.iter().map(|f| f.name()).collect();
        assert_eq!(names.len(), 3);
    }

    #[test]
    fn ji_flavor_default_is_five_limit() {
        assert_eq!(JIFlavor::default(), JIFlavor::FiveLimit);
    }

    // ── scale_quantization_table tests ───────────────────────────

    #[test]
    fn quant_c_major() {
        // C Major: C D E F G A B  (pc 0,2,4,5,7,9,11)
        let t = scale_quantization_table(Key::C, Scale::Major);
        // In-scale notes: correction = 0
        assert_eq!(t[0], 0.0); // C
        assert_eq!(t[2], 0.0); // D
        assert_eq!(t[4], 0.0); // E
        assert_eq!(t[5], 0.0); // F
        assert_eq!(t[7], 0.0); // G
        assert_eq!(t[9], 0.0); // A
        assert_eq!(t[11], 0.0); // B
        // Out-of-scale: snap to nearest
        assert_eq!(t[1], -1.0); // C# → C (down 1)
        assert_eq!(t[3], -1.0); // D# → D (down 1, equidistant picks down)
        assert_eq!(t[6], -1.0); // F# → F (down 1)
        assert_eq!(t[8], -1.0); // G# → G (down 1)
        assert_eq!(t[10], -1.0); // A# → A (down 1)
    }

    #[test]
    fn quant_a_minor() {
        // A Minor: A B C D E F G  (pc 9,11,0,2,4,5,7)
        let t = scale_quantization_table(Key::A, Scale::Minor);
        assert_eq!(t[9], 0.0); // A
        assert_eq!(t[11], 0.0); // B
        assert_eq!(t[0], 0.0); // C
        assert_eq!(t[2], 0.0); // D
        assert_eq!(t[4], 0.0); // E
        assert_eq!(t[5], 0.0); // F
        assert_eq!(t[7], 0.0); // G
        // Out-of-scale
        assert_eq!(t[1], -1.0); // C# → C
        assert_eq!(t[3], -1.0); // D# → D
        assert_eq!(t[6], -1.0); // F# → F
        assert_eq!(t[8], -1.0); // G# → G
        assert_eq!(t[10], -1.0); // A# → A
    }

    #[test]
    fn quant_chromatic_all_zeros() {
        let t = scale_quantization_table(Key::C, Scale::Chromatic);
        assert_eq!(t, [0.0; 12]);
    }

    #[test]
    fn quant_chromatic_any_key() {
        // Chromatic in any key should be all zeros
        for key in Key::ALL {
            let t = scale_quantization_table(key, Scale::Chromatic);
            assert_eq!(t, [0.0; 12], "Chromatic in {:?}", key);
        }
    }

    #[test]
    fn quant_fs_pentatonic() {
        // F# Pentatonic: F# G# A# C# D#  (pc 6,8,10,1,3)
        let t = scale_quantization_table(Key::Fs, Scale::Pentatonic);
        assert_eq!(t[6], 0.0); // F#
        assert_eq!(t[8], 0.0); // G#
        assert_eq!(t[10], 0.0); // A#
        assert_eq!(t[1], 0.0); // C#
        assert_eq!(t[3], 0.0); // D#
        // Gaps are wider in pentatonic, test some
        assert_eq!(t[0], 1.0); // C → C# (up 1)
        assert_eq!(t[2], -1.0); // D → C# (down 1)
        assert_eq!(t[7], -1.0); // G → F# (down 1)
    }

    #[test]
    fn quant_c_blues() {
        // C Blues: C Eb F F# G Bb  (pc 0,3,5,6,7,10)
        let t = scale_quantization_table(Key::C, Scale::Blues);
        assert_eq!(t[0], 0.0); // C
        assert_eq!(t[3], 0.0); // Eb
        assert_eq!(t[5], 0.0); // F
        assert_eq!(t[6], 0.0); // F#
        assert_eq!(t[7], 0.0); // G
        assert_eq!(t[10], 0.0); // Bb
        // Out-of-scale
        assert_eq!(t[1], -1.0); // C# → C
        assert_eq!(t[2], 1.0); // D → Eb
        assert_eq!(t[4], -1.0); // E → Eb (down 1, equidistant picks down)
        assert_eq!(t[8], -1.0); // G# → G
    }

    #[test]
    fn quant_all_corrections_within_range() {
        for key in Key::ALL {
            for scale in Scale::ALL {
                let t = scale_quantization_table(key, scale);
                for (pc, &corr) in t.iter().enumerate() {
                    assert!(
                        (-6.0..=6.0).contains(&corr),
                        "key={:?} scale={:?} pc={} corr={}",
                        key,
                        scale,
                        pc,
                        corr
                    );
                }
            }
        }
    }

    // ── tuning_ref_offset tests ──────────────────────────────────

    #[test]
    fn tuning_ref_440_is_zero() {
        let offset = tuning_ref_offset(440.0);
        assert!((offset).abs() < 1e-6);
    }

    #[test]
    fn tuning_ref_432() {
        let offset = tuning_ref_offset(432.0);
        // 432 Hz is about -31.77 cents = -0.3177 semitones below 440
        assert!((offset - (-0.31766)).abs() < 0.001, "offset={}", offset);
    }

    #[test]
    fn tuning_ref_880_is_twelve() {
        let offset = tuning_ref_offset(880.0);
        assert!((offset - 12.0).abs() < 1e-6);
    }

    // -- key_root_freq tests --------------------------------------

    #[test]
    fn key_root_freq_a_matches_tuning_a4() {
        let f = key_root_freq(Key::A, 440.0);
        assert!((f - 440.0).abs() < 1e-6);
    }

    #[test]
    fn key_root_freq_c_at_440() {
        let f = key_root_freq(Key::C, 440.0);
        assert!((f - 261.62555).abs() < 0.001, "f={}", f);
    }

    #[test]
    fn key_root_freq_scales_with_tuning_a4() {
        let f = key_root_freq(Key::A, 432.0);
        assert!((f - 432.0).abs() < 1e-6);
    }
}
