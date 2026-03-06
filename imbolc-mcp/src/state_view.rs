//! Format DAW state concisely for Claude readability.

use imbolc_types::ipc::StatusInfo;
use imbolc_types::{EffectSlot, ProcessingStage, Track, TrackState};

/// Format a status summary.
pub fn format_status(s: &StatusInfo) -> String {
    let playing = if s.playing { "playing" } else { "stopped" };
    let project = s.project_path.as_deref().unwrap_or("(unsaved)");
    format!(
        "BPM: {} | Key: {} {} | Tracks: {} | {} | Project: {}",
        s.bpm, s.key, s.scale, s.track_count, playing, project
    )
}

/// Format all tracks as a compact summary.
pub fn format_tracks(tracks: &TrackState) -> String {
    if tracks.tracks.is_empty() {
        return "No tracks.".to_string();
    }
    let mut lines = Vec::new();
    for (i, track) in tracks.tracks.iter().enumerate() {
        lines.push(format_track_summary(i, track));
    }
    lines.join("\n")
}

/// One-line track summary.
fn format_track_summary(index: usize, t: &Track) -> String {
    let cs = &t.channel_strip;
    let mute = if cs.mute { " [MUTED]" } else { "" };
    let solo = if cs.solo { " [SOLO]" } else { "" };
    format!(
        "Track {} \"{}\" ({}) — level: {:.2}, pan: {:.1}{}{}",
        index + 1,
        t.name,
        t.source.name(),
        cs.level,
        cs.pan,
        mute,
        solo,
    )
}

/// Format full track detail.
pub fn format_track_detail(t: &Track) -> String {
    let cs = &t.channel_strip;
    let mut lines = vec![format!(
        "Track \"{}\" (id: {}, source: {})",
        t.name,
        t.id,
        t.source.name()
    )];

    lines.push(format!(
        "  Mixer: level={:.2}, pan={:.1}, mute={}, solo={}",
        cs.level, cs.pan, cs.mute, cs.solo
    ));

    // Filter
    if let Some(ProcessingStage::Filter(f)) = cs
        .processing_chain
        .iter()
        .find(|s| matches!(s, ProcessingStage::Filter(_)))
    {
        lines.push(format!(
            "  Filter: {} @ {:.0}Hz, res {:.2}{}",
            f.filter_type.name(),
            f.cutoff.value,
            f.resonance.value,
            if f.enabled { "" } else { " (bypassed)" }
        ));
    }

    // Effects
    let effects: Vec<&EffectSlot> = cs
        .processing_chain
        .iter()
        .filter_map(|s| {
            if let ProcessingStage::Effect(e) = s {
                Some(e)
            } else {
                None
            }
        })
        .collect();
    if !effects.is_empty() {
        let names: Vec<String> = effects
            .iter()
            .map(|e| {
                let bypassed = if e.enabled { "" } else { " (off)" };
                format!("{}{}", e.effect_type.name(), bypassed)
            })
            .collect();
        lines.push(format!("  Effects: [{}]", names.join(", ")));
    }

    // Modulation
    let lfo = &t.modulation.lfo;
    let env = &t.modulation.amp_envelope;
    lines.push(format!(
        "  LFO: {} | Envelope: A={:.3} D={:.2} S={:.2} R={:.2}",
        if lfo.enabled { "on" } else { "off" },
        env.attack,
        env.decay,
        env.sustain,
        env.release,
    ));

    // Source params
    if !t.source_params.is_empty() {
        let params: Vec<String> = t
            .source_params
            .iter()
            .map(|p| format!("{}={:?}", p.name, p.value))
            .collect();
        lines.push(format!("  Params: {}", params.join(", ")));
    }

    lines.join("\n")
}
