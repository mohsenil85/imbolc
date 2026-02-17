use std::path::PathBuf;

use imbolc_types::*;

/// Split input into tokens, respecting double-quoted strings.
///
/// Splits on whitespace but preserves spaces inside `"..."`. Quotes are stripped
/// from the output. Returns `Err` for unclosed quotes.
pub fn tokenize(input: &str) -> Result<Vec<String>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        if ch.is_whitespace() {
            chars.next();
            continue;
        }
        if ch == '"' {
            chars.next(); // consume opening quote
            let mut token = String::new();
            loop {
                match chars.next() {
                    Some('"') => break,
                    Some(c) => token.push(c),
                    None => return Err("unclosed quote".to_string()),
                }
            }
            tokens.push(token);
        } else {
            let mut token = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                token.push(c);
                chars.next();
            }
            tokens.push(token);
        }
    }

    Ok(tokens)
}

/// Trait for types that can be parsed from REPL command arguments.
pub trait ReplParseable: Sized {
    fn parse_repl(s: &str) -> Result<Self, String>;
    fn type_hint() -> &'static str;
}

// ============================================================================
// Primitive impls
// ============================================================================

impl ReplParseable for f32 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid float: {s}"))
    }
    fn type_hint() -> &'static str {
        "<float>"
    }
}

impl ReplParseable for u8 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid u8: {s}"))
    }
    fn type_hint() -> &'static str {
        "<u8>"
    }
}

impl ReplParseable for u16 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid u16: {s}"))
    }
    fn type_hint() -> &'static str {
        "<u16>"
    }
}

impl ReplParseable for u32 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid u32: {s}"))
    }
    fn type_hint() -> &'static str {
        "<u32>"
    }
}

impl ReplParseable for i8 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid i8: {s}"))
    }
    fn type_hint() -> &'static str {
        "<i8>"
    }
}

impl ReplParseable for i32 {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid i32: {s}"))
    }
    fn type_hint() -> &'static str {
        "<int>"
    }
}

impl ReplParseable for usize {
    fn parse_repl(s: &str) -> Result<Self, String> {
        s.parse().map_err(|_| format!("invalid index: {s}"))
    }
    fn type_hint() -> &'static str {
        "<index>"
    }
}

impl ReplParseable for bool {
    fn parse_repl(s: &str) -> Result<Self, String> {
        match s {
            "true" | "1" | "yes" | "on" => Ok(true),
            "false" | "0" | "no" | "off" => Ok(false),
            _ => Err(format!("invalid bool: {s} (expected true/false)")),
        }
    }
    fn type_hint() -> &'static str {
        "<bool>"
    }
}

impl ReplParseable for String {
    fn parse_repl(s: &str) -> Result<Self, String> {
        Ok(s.to_string())
    }
    fn type_hint() -> &'static str {
        "<text>"
    }
}

impl ReplParseable for char {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let mut chars = s.chars();
        match (chars.next(), chars.next()) {
            (Some(c), None) => Ok(c),
            _ => Err(format!("expected single character: {s}")),
        }
    }
    fn type_hint() -> &'static str {
        "<char>"
    }
}

impl ReplParseable for PathBuf {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let expanded = if let Some(rest) = s.strip_prefix('~') {
            if let Some(home) = dirs::home_dir() {
                home.join(rest.trim_start_matches('/'))
            } else {
                PathBuf::from(s)
            }
        } else {
            PathBuf::from(s)
        };
        Ok(expanded)
    }
    fn type_hint() -> &'static str {
        "<path>"
    }
}

// ============================================================================
// Newtype ID impls
// ============================================================================

impl ReplParseable for InstrumentId {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let id: u32 = s
            .parse()
            .map_err(|_| format!("invalid instrument id: {s}"))?;
        Ok(InstrumentId::new(id))
    }
    fn type_hint() -> &'static str {
        "<instrument_id>"
    }
}

impl ReplParseable for EffectId {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let id: u32 = s.parse().map_err(|_| format!("invalid effect id: {s}"))?;
        Ok(EffectId::new(id))
    }
    fn type_hint() -> &'static str {
        "<effect_id>"
    }
}

impl ReplParseable for BusId {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let id: u8 = s.parse().map_err(|_| format!("invalid bus id: {s}"))?;
        if id == 0 {
            return Err("bus id must be >= 1".to_string());
        }
        Ok(BusId::new(id))
    }
    fn type_hint() -> &'static str {
        "<bus_id>"
    }
}

impl ReplParseable for ParamIndex {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let idx: usize = s.parse().map_err(|_| format!("invalid param index: {s}"))?;
        Ok(ParamIndex::new(idx))
    }
    fn type_hint() -> &'static str {
        "<param_idx>"
    }
}

impl ReplParseable for GenVoiceId {
    fn parse_repl(s: &str) -> Result<Self, String> {
        let id: u32 = s.parse().map_err(|_| format!("invalid voice id: {s}"))?;
        Ok(GenVoiceId::new(id))
    }
    fn type_hint() -> &'static str {
        "<voice_id>"
    }
}

// ============================================================================
// Enum impls
// ============================================================================

impl ReplParseable for SourceType {
    fn parse_repl(s: &str) -> Result<Self, String> {
        SourceType::from_short_name(s)
            .ok_or_else(|| format!("unknown source type: {s} (try: saw, sin, sqr, tri, kick, ...)"))
    }
    fn type_hint() -> &'static str {
        "<source>"
    }
}

impl ReplParseable for EffectType {
    fn parse_repl(s: &str) -> Result<Self, String> {
        EffectType::from_name(s)
            .ok_or_else(|| format!("unknown effect type: {s} (try: delay, reverb, chorus, ...)"))
    }
    fn type_hint() -> &'static str {
        "<effect>"
    }
}

impl ReplParseable for FilterType {
    fn parse_repl(s: &str) -> Result<Self, String> {
        FilterType::from_name(s)
            .ok_or_else(|| format!("unknown filter type: {s} (try: lpf, hpf, bpf, notch, ...)"))
    }
    fn type_hint() -> &'static str {
        "<filter>"
    }
}

impl ReplParseable for LfoShape {
    fn parse_repl(s: &str) -> Result<Self, String> {
        LfoShape::from_name(s)
            .ok_or_else(|| format!("unknown LFO shape: {s} (try: sine, square, saw, triangle)"))
    }
    fn type_hint() -> &'static str {
        "<lfo_shape>"
    }
}

impl ReplParseable for ParameterTarget {
    fn parse_repl(s: &str) -> Result<Self, String> {
        ParameterTarget::from_short_name(s)
            .ok_or_else(|| format!("unknown parameter target: {s} (try: Level, Pan, FltCt, ...)"))
    }
    fn type_hint() -> &'static str {
        "<target>"
    }
}

impl ReplParseable for CurveType {
    fn parse_repl(s: &str) -> Result<Self, String> {
        CurveType::from_name(s).ok_or_else(|| {
            format!("unknown curve type: {s} (try: linear, exponential, step, scurve)")
        })
    }
    fn type_hint() -> &'static str {
        "<curve>"
    }
}

impl ReplParseable for EqualizerParamKind {
    fn parse_repl(s: &str) -> Result<Self, String> {
        match s {
            "freq" => Ok(EqualizerParamKind::Freq),
            "gain" => Ok(EqualizerParamKind::Gain),
            "q" => Ok(EqualizerParamKind::Q),
            "on" | "enabled" => Ok(EqualizerParamKind::Enabled),
            _ => Err(format!("unknown eq param: {s} (try: freq, gain, q, on)")),
        }
    }
    fn type_hint() -> &'static str {
        "<eq_param>"
    }
}

impl ReplParseable for SwingGrid {
    fn parse_repl(s: &str) -> Result<Self, String> {
        SwingGrid::from_name(s)
            .ok_or_else(|| format!("unknown swing grid: {s} (try: 8ths, 16ths, both)"))
    }
    fn type_hint() -> &'static str {
        "<swing_grid>"
    }
}

impl ReplParseable for MixerSelection {
    fn parse_repl(s: &str) -> Result<Self, String> {
        if s == "master" {
            return Ok(MixerSelection::Master);
        }
        if let Some(rest) = s.strip_prefix("instrument:") {
            let idx: usize = rest
                .parse()
                .map_err(|_| format!("invalid instrument index: {rest}"))?;
            return Ok(MixerSelection::Instrument(idx));
        }
        if let Some(rest) = s.strip_prefix("bus:") {
            let id: u8 = rest
                .parse()
                .map_err(|_| format!("invalid bus id: {rest}"))?;
            return Ok(MixerSelection::Bus(BusId::new(id)));
        }
        if let Some(rest) = s.strip_prefix("group:") {
            let id: u32 = rest
                .parse()
                .map_err(|_| format!("invalid group id: {rest}"))?;
            return Ok(MixerSelection::Group(id));
        }
        Err(format!(
            "invalid mixer selection: {s} (try: master, instrument:0, bus:1, group:0)"
        ))
    }
    fn type_hint() -> &'static str {
        "<selection>"
    }
}

impl ReplParseable for VstTarget {
    fn parse_repl(s: &str) -> Result<Self, String> {
        if s == "source" {
            return Ok(VstTarget::Source);
        }
        if let Some(rest) = s.strip_prefix("effect:") {
            let id: u32 = rest
                .parse()
                .map_err(|_| format!("invalid effect id: {rest}"))?;
            return Ok(VstTarget::Effect(EffectId::new(id)));
        }
        Err(format!("invalid vst target: {s} (try: source, effect:0)"))
    }
    fn type_hint() -> &'static str {
        "<vst_target>"
    }
}

impl ReplParseable for VstPluginKind {
    fn parse_repl(s: &str) -> Result<Self, String> {
        match s {
            "instrument" => Ok(VstPluginKind::Instrument),
            "effect" => Ok(VstPluginKind::Effect),
            _ => Err(format!("invalid vst kind: {s} (try: instrument, effect)")),
        }
    }
    fn type_hint() -> &'static str {
        "<vst_kind>"
    }
}

impl ReplParseable for Key {
    fn parse_repl(s: &str) -> Result<Self, String> {
        Key::from_name(s).ok_or_else(|| {
            format!("unknown key: {s} (try: C, C#, D, D#, E, F, F#, G, G#, A, A#, B)")
        })
    }
    fn type_hint() -> &'static str {
        "<key>"
    }
}

impl ReplParseable for Scale {
    fn parse_repl(s: &str) -> Result<Self, String> {
        Scale::from_name(s)
            .ok_or_else(|| format!("unknown scale: {s} (try: Major, Minor, Dorian, ...)"))
    }
    fn type_hint() -> &'static str {
        "<scale>"
    }
}

impl ReplParseable for Tuning {
    fn parse_repl(s: &str) -> Result<Self, String> {
        Tuning::from_name(s).ok_or_else(|| {
            format!("unknown tuning: {s} (try: 12-TET, ScaleJI, ChordJI, AdaptiveJI, GlobalJI)")
        })
    }
    fn type_hint() -> &'static str {
        "<tuning>"
    }
}

impl ReplParseable for JIFlavor {
    fn parse_repl(s: &str) -> Result<Self, String> {
        JIFlavor::from_name(s)
            .ok_or_else(|| format!("unknown JI flavor: {s} (try: 5-Limit, 7-Limit, Pythagorean)"))
    }
    fn type_hint() -> &'static str {
        "<ji_flavor>"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_primitives() {
        assert_eq!(f32::parse_repl("2.5").unwrap(), 2.5);
        assert_eq!(u8::parse_repl("42").unwrap(), 42);
        assert_eq!(u32::parse_repl("1000").unwrap(), 1000);
        assert_eq!(i8::parse_repl("-5").unwrap(), -5);
        assert_eq!(usize::parse_repl("0").unwrap(), 0);
        assert!(bool::parse_repl("true").unwrap());
        assert!(!bool::parse_repl("off").unwrap());
    }

    #[test]
    fn parse_ids() {
        assert_eq!(InstrumentId::parse_repl("7").unwrap(), InstrumentId::new(7));
        assert_eq!(EffectId::parse_repl("3").unwrap(), EffectId::new(3));
        assert_eq!(BusId::parse_repl("1").unwrap(), BusId::new(1));
        assert!(BusId::parse_repl("0").is_err());
        assert_eq!(ParamIndex::parse_repl("2").unwrap(), ParamIndex::new(2));
        assert_eq!(GenVoiceId::parse_repl("5").unwrap(), GenVoiceId::new(5));
    }

    #[test]
    fn parse_source_type() {
        assert_eq!(SourceType::parse_repl("saw").unwrap(), SourceType::Saw);
        assert_eq!(SourceType::parse_repl("kick").unwrap(), SourceType::Kick);
        assert!(SourceType::parse_repl("nonexistent").is_err());
    }

    #[test]
    fn parse_effect_type() {
        assert_eq!(EffectType::parse_repl("delay").unwrap(), EffectType::Delay);
        assert_eq!(
            EffectType::parse_repl("reverb").unwrap(),
            EffectType::Reverb
        );
    }

    #[test]
    fn parse_filter_type() {
        assert_eq!(FilterType::parse_repl("lpf").unwrap(), FilterType::Lpf);
        assert_eq!(FilterType::parse_repl("hpf").unwrap(), FilterType::Hpf);
    }

    #[test]
    fn parse_lfo_shape() {
        assert_eq!(LfoShape::parse_repl("Sine").unwrap(), LfoShape::Sine);
        assert_eq!(LfoShape::parse_repl("Saw").unwrap(), LfoShape::Saw);
    }

    #[test]
    fn parse_curve_type() {
        assert_eq!(CurveType::parse_repl("linear").unwrap(), CurveType::Linear);
        assert_eq!(CurveType::parse_repl("step").unwrap(), CurveType::Step);
    }

    #[test]
    fn parse_mixer_selection() {
        assert_eq!(
            MixerSelection::parse_repl("master").unwrap(),
            MixerSelection::Master
        );
        assert_eq!(
            MixerSelection::parse_repl("instrument:0").unwrap(),
            MixerSelection::Instrument(0)
        );
        assert_eq!(
            MixerSelection::parse_repl("bus:1").unwrap(),
            MixerSelection::Bus(BusId::new(1))
        );
    }

    #[test]
    fn parse_path_tilde_expansion() {
        let path = PathBuf::parse_repl("~/test.wav").unwrap();
        assert!(!path.to_str().unwrap().starts_with('~'));
    }

    #[test]
    fn parse_key() {
        assert_eq!(Key::parse_repl("C").unwrap(), Key::C);
        assert_eq!(Key::parse_repl("C#").unwrap(), Key::Cs);
        assert_eq!(Key::parse_repl("F#").unwrap(), Key::Fs);
    }

    #[test]
    fn parse_scale() {
        assert_eq!(Scale::parse_repl("Major").unwrap(), Scale::Major);
        assert_eq!(Scale::parse_repl("Minor").unwrap(), Scale::Minor);
    }

    #[test]
    fn tokenize_simple() {
        assert_eq!(
            tokenize("instrument add saw").unwrap(),
            vec!["instrument", "add", "saw"]
        );
    }

    #[test]
    fn tokenize_quoted() {
        assert_eq!(
            tokenize(r#"session create-checkpoint "pre-mix v2""#).unwrap(),
            vec!["session", "create-checkpoint", "pre-mix v2"]
        );
    }

    #[test]
    fn tokenize_unclosed_quote() {
        assert!(tokenize(r#"session create-checkpoint "unclosed"#).is_err());
    }

    #[test]
    fn tokenize_empty() {
        assert_eq!(tokenize("").unwrap(), Vec::<String>::new());
        assert_eq!(tokenize("   ").unwrap(), Vec::<String>::new());
    }

    #[test]
    fn tokenize_extra_whitespace() {
        assert_eq!(
            tokenize("  instrument   add   saw  ").unwrap(),
            vec!["instrument", "add", "saw"]
        );
    }
}
