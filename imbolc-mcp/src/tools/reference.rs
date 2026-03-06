//! Reference tools — work standalone without a running DAW.

use imbolc_types::state::track::meta::*;

/// Format the full instrument catalog for Claude.
pub fn list_instruments() -> String {
    let mut out = String::new();
    let mut current_cat = "";
    for m in INSTRUMENT_CATALOG {
        if m.category != current_cat {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("## {}\n", m.category));
            current_cat = m.category;
        }
        out.push_str(&format!(
            "- **{}** (`{}`) — {}\n  Tags: {}\n  Use: {}\n",
            m.name,
            m.short_name,
            m.description,
            m.tags.join(", "),
            m.use_cases,
        ));
        if !m.aliases.is_empty() {
            out.push_str(&format!("  Aliases: {}\n", m.aliases.join(", ")));
        }
    }
    out
}

/// Format the full effect catalog for Claude.
pub fn list_effects() -> String {
    let mut out = String::new();
    let mut current_cat = "";
    for m in EFFECT_CATALOG {
        if m.category != current_cat {
            if !out.is_empty() {
                out.push('\n');
            }
            out.push_str(&format!("## {}\n", m.category));
            current_cat = m.category;
        }
        out.push_str(&format!(
            "- **{}** — {}\n  Tags: {}\n  Use: {}\n",
            m.name,
            m.description,
            m.tags.join(", "),
            m.use_cases,
        ));
        if !m.aliases.is_empty() {
            out.push_str(&format!("  Aliases: {}\n", m.aliases.join(", ")));
        }
    }
    out
}

/// Format the full filter catalog for Claude.
pub fn list_filters() -> String {
    let mut out = String::new();
    for m in FILTER_CATALOG {
        out.push_str(&format!(
            "- **{}** — {}\n  Tags: {}\n",
            m.name,
            m.description,
            m.tags.join(", "),
        ));
    }
    out
}

/// Describe a single instrument by name (natural language lookup).
pub fn describe_instrument(name: &str) -> String {
    match imbolc_types::SourceType::from_natural_language(name) {
        Some(st) => match st.meta() {
            Some(m) => {
                let mut out = format!(
                    "**{}** (`{}`)\nCategory: {}\n{}\n\nUse cases: {}\nTags: {}",
                    m.name,
                    m.short_name,
                    m.category,
                    m.description,
                    m.use_cases,
                    m.tags.join(", "),
                );
                if !m.aliases.is_empty() {
                    out.push_str(&format!("\nAliases: {}", m.aliases.join(", ")));
                }
                // Add default params
                let params = st.default_params();
                if !params.is_empty() {
                    out.push_str("\n\nParameters:");
                    for p in &params {
                        out.push_str(&format!(
                            "\n  {} = {} (range: {} to {})",
                            p.name,
                            p.value.to_f32(),
                            p.min,
                            p.max
                        ));
                    }
                }
                out
            }
            None => format!(
                "'{}' is a custom/VST instrument with no built-in metadata.",
                name
            ),
        },
        None => format!(
            "Unknown instrument: '{}'. Use list_instruments to see all available.",
            name
        ),
    }
}

/// Describe a single effect by name (natural language lookup).
pub fn describe_effect(name: &str) -> String {
    match imbolc_types::EffectType::from_natural_language(name) {
        Some(et) => match et.meta() {
            Some(m) => {
                let mut out = format!(
                    "**{}**\nCategory: {}\n{}\n\nUse cases: {}\nTags: {}",
                    m.name,
                    m.category,
                    m.description,
                    m.use_cases,
                    m.tags.join(", "),
                );
                if !m.aliases.is_empty() {
                    out.push_str(&format!("\nAliases: {}", m.aliases.join(", ")));
                }
                // Add default params
                let params = et.default_params();
                if !params.is_empty() {
                    out.push_str("\n\nParameters:");
                    for p in &params {
                        out.push_str(&format!(
                            "\n  {} = {} (range: {} to {})",
                            p.name,
                            p.value.to_f32(),
                            p.min,
                            p.max
                        ));
                    }
                }
                out
            }
            None => format!("'{}' is a VST effect with no built-in metadata.", name),
        },
        None => format!(
            "Unknown effect: '{}'. Use list_effects to see all available.",
            name
        ),
    }
}
