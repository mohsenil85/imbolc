use std::path::PathBuf;

use serde::Deserialize;

use crate::state::music::{JIFlavor, Key, Scale, Tuning};
use crate::state::KeyboardLayout;
use crate::state::MusicalSettings;

const DEFAULT_CONFIG: &str = include_str!("../config.toml");

#[derive(Deserialize, Default)]
struct ConfigFile {
    #[serde(default)]
    defaults: DefaultsConfig,
    #[serde(default)]
    runtime: RuntimeConfig,
}

#[derive(Deserialize, Default)]
struct DefaultsConfig {
    bpm: Option<u16>,
    key: Option<String>,
    scale: Option<String>,
    tuning_a4: Option<f32>,
    time_signature: Option<[u8; 2]>,
    snap: Option<bool>,
    keyboard_layout: Option<String>,
    bus_count: Option<u8>,
    tuning: Option<String>,
    ji_flavor: Option<String>,
}

#[derive(Deserialize, Default)]
struct RuntimeConfig {
    autosave: Option<bool>,
    autosave_interval_minutes: Option<u64>,
    theme: Option<String>,
}

pub struct Config {
    defaults: DefaultsConfig,
    runtime: RuntimeConfig,
}

impl Config {
    pub fn load() -> Self {
        let mut base: ConfigFile =
            toml::from_str(DEFAULT_CONFIG).expect("Failed to parse embedded config.toml");

        if let Some(path) = user_config_path() {
            if path.exists() {
                match std::fs::read_to_string(&path) {
                    Ok(contents) => match toml::from_str::<ConfigFile>(&contents) {
                        Ok(user) => {
                            merge_defaults(&mut base.defaults, user.defaults);
                            merge_runtime(&mut base.runtime, user.runtime);
                        }
                        Err(e) => {
                            log::warn!(target: "config", "ignoring malformed config {}: {}", path.display(), e)
                        }
                    },
                    Err(e) => {
                        log::warn!(target: "config", "could not read config {}: {}", path.display(), e)
                    }
                }
            }
        }

        Config {
            defaults: base.defaults,
            runtime: base.runtime,
        }
    }

    pub fn keyboard_layout(&self) -> KeyboardLayout {
        self.defaults
            .keyboard_layout
            .as_deref()
            .and_then(parse_keyboard_layout)
            .unwrap_or_default()
    }

    /// Get the default number of mixing buses for new projects
    pub fn default_bus_count(&self) -> u8 {
        use crate::state::session::DEFAULT_BUS_COUNT;
        self.defaults.bus_count.unwrap_or(DEFAULT_BUS_COUNT)
    }

    pub fn defaults(&self) -> MusicalSettings {
        let fallback = MusicalSettings::default();
        MusicalSettings {
            bpm: self.defaults.bpm.unwrap_or(fallback.bpm),
            key: self
                .defaults
                .key
                .as_deref()
                .and_then(parse_key)
                .unwrap_or(fallback.key),
            scale: self
                .defaults
                .scale
                .as_deref()
                .and_then(parse_scale)
                .unwrap_or(fallback.scale),
            tuning_a4: self.defaults.tuning_a4.unwrap_or(fallback.tuning_a4),
            time_signature: self
                .defaults
                .time_signature
                .map(|ts| (ts[0], ts[1]))
                .unwrap_or(fallback.time_signature),
            snap: self.defaults.snap.unwrap_or(fallback.snap),
            tuning: self
                .defaults
                .tuning
                .as_deref()
                .and_then(parse_tuning)
                .unwrap_or(fallback.tuning),
            ji_flavor: self
                .defaults
                .ji_flavor
                .as_deref()
                .and_then(parse_ji_flavor)
                .unwrap_or(fallback.ji_flavor),
        }
    }

    /// Whether periodic autosave snapshots are enabled.
    pub fn autosave_enabled(&self) -> bool {
        self.runtime.autosave.unwrap_or(true)
    }

    /// Autosave interval in minutes (clamped to 1..10080).
    pub fn autosave_interval_minutes(&self) -> u64 {
        self.runtime
            .autosave_interval_minutes
            .unwrap_or(2)
            .clamp(1, 10_080)
    }

    /// Get the configured theme, checking built-ins then user themes, falling back to Minimal.
    pub fn theme(&self) -> imbolc_types::state::theme::Theme {
        let Some(name) = self.runtime.theme.as_deref() else {
            return imbolc_types::state::theme::Theme::default();
        };
        // Check built-ins first
        if let Some(t) = imbolc_types::state::theme::Theme::from_name(name) {
            return t;
        }
        // Check user themes
        for t in load_user_themes() {
            if t.name.eq_ignore_ascii_case(name) {
                return t;
            }
        }
        imbolc_types::state::theme::Theme::default()
    }

    /// Persist the theme name to the user config file.
    /// Creates `~/.config/imbolc/config.toml` if it doesn't exist.
    pub fn save_theme(name: &str) {
        let Some(path) = user_config_path() else {
            return;
        };
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        // Read existing file or start fresh
        let contents = std::fs::read_to_string(&path).unwrap_or_default();

        // Update or insert the theme line under [runtime]
        let new_contents = update_runtime_theme(&contents, name);
        let _ = std::fs::write(&path, new_contents);
    }
}

fn user_config_path() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("imbolc").join("config.toml"))
}

/// Path to user-defined theme files: `~/.config/imbolc/themes/`
pub fn user_themes_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("imbolc").join("themes"))
}

// ============================================================================
// User-defined TOML theme files
// ============================================================================

use imbolc_types::state::theme::{Theme, ThemeColor};

/// Deserialization-only struct for user theme files.
/// All color fields are optional to support partial overrides via `extends`.
#[derive(Deserialize)]
struct ThemeFile {
    name: String,
    #[serde(default)]
    extends: Option<String>,

    // Base colors
    background: Option<ThemeColor>,
    foreground: Option<ThemeColor>,
    border: Option<ThemeColor>,

    // Accent colors
    accent: Option<ThemeColor>,
    accent_secondary: Option<ThemeColor>,
    dim: Option<ThemeColor>,

    // Semantic UI colors
    selection_bg: Option<ThemeColor>,
    selection_fg: Option<ThemeColor>,
    muted: Option<ThemeColor>,
    error: Option<ThemeColor>,
    warning: Option<ThemeColor>,
    success: Option<ThemeColor>,

    // Track type indicator colors
    mute_color: Option<ThemeColor>,
    solo_color: Option<ThemeColor>,
    bus_color: Option<ThemeColor>,
    group_color: Option<ThemeColor>,
    master_color: Option<ThemeColor>,

    // Module type colors
    osc_color: Option<ThemeColor>,
    filter_color: Option<ThemeColor>,
    env_color: Option<ThemeColor>,
    lfo_color: Option<ThemeColor>,
    fx_color: Option<ThemeColor>,
    sample_color: Option<ThemeColor>,
    midi_color: Option<ThemeColor>,
    audio_in_color: Option<ThemeColor>,
    eq_color: Option<ThemeColor>,
    custom_color: Option<ThemeColor>,
    kit_color: Option<ThemeColor>,
    bus_in_color: Option<ThemeColor>,
    vst_color: Option<ThemeColor>,

    // Meter colors
    meter_low: Option<ThemeColor>,
    meter_mid: Option<ThemeColor>,
    meter_high: Option<ThemeColor>,

    // Waveform gradient
    waveform_gradient: Option<[ThemeColor; 4]>,

    // Status colors
    playing: Option<ThemeColor>,
    recording: Option<ThemeColor>,
    armed: Option<ThemeColor>,
}

impl ThemeFile {
    /// Overlay non-None fields onto a base theme, producing a complete Theme.
    fn apply_to(self, base: &Theme) -> Theme {
        Theme {
            name: self.name,
            background: self.background.unwrap_or(base.background),
            foreground: self.foreground.unwrap_or(base.foreground),
            border: self.border.unwrap_or(base.border),
            accent: self.accent.unwrap_or(base.accent),
            accent_secondary: self.accent_secondary.unwrap_or(base.accent_secondary),
            dim: self.dim.unwrap_or(base.dim),
            selection_bg: self.selection_bg.unwrap_or(base.selection_bg),
            selection_fg: self.selection_fg.unwrap_or(base.selection_fg),
            muted: self.muted.unwrap_or(base.muted),
            error: self.error.unwrap_or(base.error),
            warning: self.warning.unwrap_or(base.warning),
            success: self.success.unwrap_or(base.success),
            mute_color: self.mute_color.unwrap_or(base.mute_color),
            solo_color: self.solo_color.unwrap_or(base.solo_color),
            bus_color: self.bus_color.unwrap_or(base.bus_color),
            group_color: self.group_color.unwrap_or(base.group_color),
            master_color: self.master_color.unwrap_or(base.master_color),
            osc_color: self.osc_color.unwrap_or(base.osc_color),
            filter_color: self.filter_color.unwrap_or(base.filter_color),
            env_color: self.env_color.unwrap_or(base.env_color),
            lfo_color: self.lfo_color.unwrap_or(base.lfo_color),
            fx_color: self.fx_color.unwrap_or(base.fx_color),
            sample_color: self.sample_color.unwrap_or(base.sample_color),
            midi_color: self.midi_color.unwrap_or(base.midi_color),
            audio_in_color: self.audio_in_color.unwrap_or(base.audio_in_color),
            eq_color: self.eq_color.unwrap_or(base.eq_color),
            custom_color: self.custom_color.unwrap_or(base.custom_color),
            kit_color: self.kit_color.unwrap_or(base.kit_color),
            bus_in_color: self.bus_in_color.unwrap_or(base.bus_in_color),
            vst_color: self.vst_color.unwrap_or(base.vst_color),
            meter_low: self.meter_low.unwrap_or(base.meter_low),
            meter_mid: self.meter_mid.unwrap_or(base.meter_mid),
            meter_high: self.meter_high.unwrap_or(base.meter_high),
            waveform_gradient: self.waveform_gradient.unwrap_or(base.waveform_gradient),
            playing: self.playing.unwrap_or(base.playing),
            recording: self.recording.unwrap_or(base.recording),
            armed: self.armed.unwrap_or(base.armed),
        }
    }
}

/// Load user-defined themes from `~/.config/imbolc/themes/*.toml`.
/// Resolves `extends` chains (built-in lookup first, then already-loaded user themes).
/// Logs warnings for malformed files. Detects circular extends.
pub fn load_user_themes() -> Vec<Theme> {
    let Some(dir) = user_themes_dir() else {
        return vec![];
    };
    if !dir.exists() {
        return vec![];
    }

    let Ok(entries) = std::fs::read_dir(&dir) else {
        return vec![];
    };

    // First pass: parse all theme files
    let mut theme_files: Vec<(String, ThemeFile)> = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("toml") {
            continue;
        }
        match std::fs::read_to_string(&path) {
            Ok(contents) => match toml::from_str::<ThemeFile>(&contents) {
                Ok(tf) => {
                    let name = tf.name.clone();
                    theme_files.push((name, tf));
                }
                Err(e) => {
                    log::warn!(target: "config", "ignoring malformed theme {}: {}", path.display(), e);
                }
            },
            Err(e) => {
                log::warn!(target: "config", "could not read theme {}: {}", path.display(), e);
            }
        }
    }

    // Second pass: resolve extends chains
    let built_ins = Theme::built_in_themes();
    let mut resolved: Vec<Theme> = Vec::new();

    // We need multiple passes since a user theme might extend another user theme
    let mut remaining = theme_files;
    let max_passes = remaining.len() + 1; // prevent infinite loops
    for _ in 0..max_passes {
        if remaining.is_empty() {
            break;
        }
        let mut still_remaining = Vec::new();
        let mut made_progress = false;
        for (name, tf) in remaining {
            match &tf.extends {
                None => {
                    // No extends — use Minimal as implicit base
                    resolved.push(tf.apply_to(&Theme::default()));
                    made_progress = true;
                }
                Some(base_name) => {
                    // Look up base in built-ins first, then in already-resolved user themes
                    let base = built_ins
                        .iter()
                        .find(|t| t.name.eq_ignore_ascii_case(base_name))
                        .or_else(|| {
                            resolved
                                .iter()
                                .find(|t| t.name.eq_ignore_ascii_case(base_name))
                        });
                    match base {
                        Some(b) => {
                            resolved.push(tf.apply_to(b));
                            made_progress = true;
                        }
                        None => {
                            still_remaining.push((name, tf));
                        }
                    }
                }
            }
        }
        remaining = still_remaining;
        if !made_progress {
            // Circular or unresolvable extends
            for (name, _) in &remaining {
                log::warn!(target: "config", "theme '{}' has unresolvable extends chain", name);
            }
            break;
        }
    }

    resolved
}

/// All available themes: built-in themes followed by user-defined themes.
pub fn all_themes() -> Vec<Theme> {
    let mut themes = Theme::built_in_themes();
    themes.extend(load_user_themes());
    themes
}

/// Serialize a theme to pretty TOML for export.
pub fn export_theme_toml(theme: &Theme) -> String {
    toml::to_string_pretty(theme).unwrap_or_default()
}

/// Write a theme TOML file to the user themes directory.
/// Creates the directory if it doesn't exist. Returns the path on success.
pub fn save_theme_file(name: &str, content: &str) -> Result<PathBuf, String> {
    let dir =
        user_themes_dir().ok_or_else(|| "could not determine config directory".to_string())?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("could not create themes directory: {}", e))?;
    let filename = name.to_lowercase().replace(' ', "-") + ".toml";
    let path = dir.join(filename);
    std::fs::write(&path, content).map_err(|e| format!("could not write theme file: {}", e))?;
    Ok(path)
}

fn merge_defaults(base: &mut DefaultsConfig, user: DefaultsConfig) {
    if user.bpm.is_some() {
        base.bpm = user.bpm;
    }
    if user.key.is_some() {
        base.key = user.key;
    }
    if user.scale.is_some() {
        base.scale = user.scale;
    }
    if user.tuning_a4.is_some() {
        base.tuning_a4 = user.tuning_a4;
    }
    if user.time_signature.is_some() {
        base.time_signature = user.time_signature;
    }
    if user.snap.is_some() {
        base.snap = user.snap;
    }
    if user.keyboard_layout.is_some() {
        base.keyboard_layout = user.keyboard_layout;
    }
    if user.bus_count.is_some() {
        base.bus_count = user.bus_count;
    }
    if user.tuning.is_some() {
        base.tuning = user.tuning;
    }
    if user.ji_flavor.is_some() {
        base.ji_flavor = user.ji_flavor;
    }
}

fn merge_runtime(base: &mut RuntimeConfig, user: RuntimeConfig) {
    if user.autosave.is_some() {
        base.autosave = user.autosave;
    }
    if user.autosave_interval_minutes.is_some() {
        base.autosave_interval_minutes = user.autosave_interval_minutes;
    }
    if user.theme.is_some() {
        base.theme = user.theme;
    }
}

/// Update or insert `theme = "..."` under `[runtime]` in a TOML string.
fn update_runtime_theme(contents: &str, theme_name: &str) -> String {
    let mut lines: Vec<String> = contents.lines().map(|l| l.to_string()).collect();
    let theme_line = format!("theme = \"{}\"", theme_name);

    // Find existing theme line under [runtime]
    let mut in_runtime = false;
    let mut found = false;
    for line in &mut lines {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_runtime = trimmed == "[runtime]";
        }
        if in_runtime && trimmed.starts_with("theme") && trimmed.contains('=') {
            *line = theme_line.clone();
            found = true;
            break;
        }
    }

    if !found {
        // Find [runtime] section or create it
        let mut runtime_idx = None;
        for (i, line) in lines.iter().enumerate() {
            if line.trim() == "[runtime]" {
                runtime_idx = Some(i);
                break;
            }
        }
        match runtime_idx {
            Some(idx) => {
                lines.insert(idx + 1, theme_line);
            }
            None => {
                if !lines.is_empty() && !lines.last().unwrap().is_empty() {
                    lines.push(String::new());
                }
                lines.push("[runtime]".to_string());
                lines.push(theme_line);
            }
        }
    }

    let mut result = lines.join("\n");
    if !result.ends_with('\n') {
        result.push('\n');
    }
    result
}

fn parse_key(s: &str) -> Option<Key> {
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

fn parse_keyboard_layout(s: &str) -> Option<KeyboardLayout> {
    match s.to_lowercase().as_str() {
        "qwerty" => Some(KeyboardLayout::Qwerty),
        "colemak" => Some(KeyboardLayout::Colemak),
        _ => None,
    }
}

fn parse_scale(s: &str) -> Option<Scale> {
    match s {
        "Major" => Some(Scale::Major),
        "Minor" => Some(Scale::Minor),
        "Dorian" => Some(Scale::Dorian),
        "Phrygian" => Some(Scale::Phrygian),
        "Lydian" => Some(Scale::Lydian),
        "Mixolydian" => Some(Scale::Mixolydian),
        "Aeolian" => Some(Scale::Aeolian),
        "Locrian" => Some(Scale::Locrian),
        "Pentatonic" => Some(Scale::Pentatonic),
        "Blues" => Some(Scale::Blues),
        "Chromatic" => Some(Scale::Chromatic),
        _ => None,
    }
}

fn parse_tuning(s: &str) -> Option<Tuning> {
    match s {
        "EqualTemperament" | "ET" | "12-TET" => Some(Tuning::EqualTemperament),
        "ScaleJI" => Some(Tuning::ScaleJI),
        "ChordJI" => Some(Tuning::ChordJI),
        "AdaptiveJI" => Some(Tuning::AdaptiveJI),
        "GlobalJI" => Some(Tuning::GlobalJI),
        _ => None,
    }
}

fn parse_ji_flavor(s: &str) -> Option<JIFlavor> {
    match s {
        "FiveLimit" | "5-Limit" | "5L" => Some(JIFlavor::FiveLimit),
        "SevenLimit" | "7-Limit" | "7L" => Some(JIFlavor::SevenLimit),
        "Pythagorean" => Some(JIFlavor::Pythagorean),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_embedded_config() {
        let config = Config::load();
        let defaults = config.defaults();
        assert_eq!(defaults.bpm, 120);
        assert_eq!(defaults.key, Key::C);
        assert_eq!(defaults.scale, Scale::Major);
        assert!((defaults.tuning_a4 - 432.0).abs() < f32::EPSILON);
        assert_eq!(defaults.time_signature, (4, 4));
        assert!(!defaults.snap);
        assert_eq!(config.keyboard_layout(), KeyboardLayout::Colemak);
        assert!(config.autosave_enabled());
        assert_eq!(config.autosave_interval_minutes(), 2);
    }

    #[test]
    fn test_parse_keys() {
        assert_eq!(parse_key("C"), Some(Key::C));
        assert_eq!(parse_key("C#"), Some(Key::Cs));
        assert_eq!(parse_key("Fs"), Some(Key::Fs));
        assert_eq!(parse_key("F#"), Some(Key::Fs));
        assert_eq!(parse_key("X"), None);
    }

    #[test]
    fn test_parse_scales() {
        assert_eq!(parse_scale("Major"), Some(Scale::Major));
        assert_eq!(parse_scale("Minor"), Some(Scale::Minor));
        assert_eq!(parse_scale("Blues"), Some(Scale::Blues));
        assert_eq!(parse_scale("Nope"), None);
    }

    #[test]
    fn test_parse_keyboard_layout() {
        assert_eq!(
            parse_keyboard_layout("qwerty"),
            Some(KeyboardLayout::Qwerty)
        );
        assert_eq!(
            parse_keyboard_layout("COLEMAK"),
            Some(KeyboardLayout::Colemak)
        );
        assert_eq!(parse_keyboard_layout("unknown"), None);
    }

    #[test]
    fn export_theme_toml_round_trip() {
        use imbolc_types::state::theme::Theme;
        let theme = Theme::dark();
        let toml_str = export_theme_toml(&theme);
        assert!(toml_str.contains("name = \"Dark\""));
        assert!(toml_str.contains("background"));
        // Should deserialize back
        let back: Theme = toml::from_str(&toml_str).unwrap();
        assert_eq!(back.name, "Dark");
        assert_eq!(back.background, theme.background);
    }

    #[test]
    fn all_themes_includes_built_ins() {
        let themes = all_themes();
        // At minimum, all 5 built-ins
        assert!(themes.len() >= 5);
        assert_eq!(themes[0].name, "Dark");
        assert_eq!(themes[1].name, "Light");
    }

    #[test]
    fn theme_file_extends_resolution() {
        use imbolc_types::state::theme::{Theme, ThemeColor};
        let tf: ThemeFile = toml::from_str(
            r#"
            name = "Neon Dark"
            extends = "Dark"
            accent = [255, 0, 255]
            "#,
        )
        .unwrap();
        let dark = Theme::dark();
        let result = tf.apply_to(&dark);
        assert_eq!(result.name, "Neon Dark");
        assert_eq!(result.accent, ThemeColor::new(255, 0, 255));
        // Inherited from Dark
        assert_eq!(result.background, dark.background);
        assert_eq!(result.foreground, dark.foreground);
    }

    #[test]
    fn save_and_load_theme_file() {
        let dir = tempfile::tempdir().unwrap();
        let themes_dir = dir.path().join("themes");
        std::fs::create_dir_all(&themes_dir).unwrap();

        let content = r#"name = "Test"
background = [10, 20, 30]
"#;
        let path = themes_dir.join("test.toml");
        std::fs::write(&path, content).unwrap();

        let tf: ThemeFile = toml::from_str(content).unwrap();
        assert_eq!(tf.name, "Test");
        assert_eq!(tf.background, Some(ThemeColor::new(10, 20, 30)));
        assert!(tf.extends.is_none());
    }
}
