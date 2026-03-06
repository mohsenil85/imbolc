//! Rich metadata for instruments, effects, and filters.
//!
//! Metadata lives in static structs enabling serialization, introspection,
//! programmatic queries, and catalog browsing (e.g. by MCP tools).

use serde::Serialize;

use super::{EffectType, FilterType, SourceType};

// ============================================================================
// Metadata structs
// ============================================================================

/// Rich metadata for an instrument source type.
#[derive(Debug, Clone, Serialize)]
pub struct InstrumentMeta {
    pub source_type: SourceType,
    pub name: &'static str,
    pub short_name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub tags: &'static [&'static str],
    pub aliases: &'static [&'static str],
    pub use_cases: &'static str,
}

/// Rich metadata for an effect type.
#[derive(Debug, Clone, Serialize)]
pub struct EffectMeta {
    pub effect_type: EffectType,
    pub name: &'static str,
    pub category: &'static str,
    pub description: &'static str,
    pub tags: &'static [&'static str],
    pub aliases: &'static [&'static str],
    pub use_cases: &'static str,
}

/// Rich metadata for a filter type.
#[derive(Debug, Clone, Serialize)]
pub struct FilterMeta {
    pub filter_type: FilterType,
    pub name: &'static str,
    pub description: &'static str,
    pub tags: &'static [&'static str],
}

// ============================================================================
// Instrument catalog
// ============================================================================

pub static INSTRUMENT_CATALOG: &[InstrumentMeta] = &[
    // --- Basic Oscillators ---
    InstrumentMeta {
        source_type: SourceType::Saw,
        name: "Saw",
        short_name: "saw",
        category: "Oscillators",
        description: "Sawtooth wave oscillator",
        tags: &["bright", "buzzy", "analog", "lead", "bass"],
        aliases: &["sawtooth"],
        use_cases: "Versatile — leads, basses, pads. Harmonically rich, responds well to filters.",
    },
    InstrumentMeta {
        source_type: SourceType::Sin,
        name: "Sine",
        short_name: "sin",
        category: "Oscillators",
        description: "Pure sine wave oscillator",
        tags: &["pure", "clean", "sub", "bass", "soft"],
        aliases: &["sine"],
        use_cases: "Sub-bass, clean tones, FM carrier. No harmonics — pure fundamental.",
    },
    InstrumentMeta {
        source_type: SourceType::Sqr,
        name: "Square",
        short_name: "sqr",
        category: "Oscillators",
        description: "Square wave oscillator",
        tags: &["hollow", "retro", "8bit", "lead", "bass"],
        aliases: &["square"],
        use_cases: "Retro/chiptune leads, hollow basses, classic synth tones.",
    },
    InstrumentMeta {
        source_type: SourceType::Tri,
        name: "Triangle",
        short_name: "tri",
        category: "Oscillators",
        description: "Triangle wave oscillator",
        tags: &["mellow", "soft", "warm", "lead"],
        aliases: &["triangle"],
        use_cases: "Mellow leads, soft pads. Warmer than sine, less harsh than saw.",
    },
    InstrumentMeta {
        source_type: SourceType::Noise,
        name: "Noise",
        short_name: "noise",
        category: "Oscillators",
        description: "Noise generator with color control",
        tags: &["noise", "texture", "percussive", "ambient"],
        aliases: &["white noise", "pink noise"],
        use_cases: "Percussion synthesis, risers, ambient textures, hi-hat-like sounds.",
    },
    InstrumentMeta {
        source_type: SourceType::Pulse,
        name: "Pulse",
        short_name: "pulse",
        category: "Oscillators",
        description: "Variable-width pulse wave oscillator",
        tags: &["nasal", "hollow", "pwm", "lead"],
        aliases: &["pwm"],
        use_cases: "PWM leads, nasal tones. Width modulation creates movement.",
    },
    InstrumentMeta {
        source_type: SourceType::SuperSaw,
        name: "SuperSaw",
        short_name: "supersaw",
        category: "Oscillators",
        description: "Detuned multi-saw oscillator",
        tags: &["fat", "wide", "trance", "pad", "lead"],
        aliases: &["super saw", "unison saw"],
        use_cases: "Massive pads, trance leads, wide stereo textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Sync,
        name: "Sync",
        short_name: "sync",
        category: "Oscillators",
        description: "Hard-sync oscillator pair",
        tags: &["aggressive", "sharp", "lead", "metallic"],
        aliases: &["hard sync"],
        use_cases:
            "Aggressive leads, metallic textures. Sync ratio sweeps create classic tearing sound.",
    },
    // --- FM/Modulation ---
    InstrumentMeta {
        source_type: SourceType::Ring,
        name: "Ring Mod",
        short_name: "ring",
        category: "FM/Modulation",
        description: "Ring modulation oscillator",
        tags: &["metallic", "inharmonic", "bell", "experimental"],
        aliases: &["ring modulation", "ring mod"],
        use_cases: "Metallic textures, bell-like inharmonic tones, experimental sound design.",
    },
    InstrumentMeta {
        source_type: SourceType::FBSin,
        name: "FB Sine",
        short_name: "fbsin",
        category: "FM/Modulation",
        description: "Feedback sine oscillator",
        tags: &["feedback", "distorted", "growl", "bass"],
        aliases: &["feedback sine"],
        use_cases: "Growling basses, distorted tones. Self-modulation creates harmonic richness.",
    },
    InstrumentMeta {
        source_type: SourceType::FM,
        name: "FM",
        short_name: "fm",
        category: "FM/Modulation",
        description: "Two-operator FM synthesis",
        tags: &["fm", "digital", "bell", "bass", "versatile"],
        aliases: &["fm synth", "frequency modulation"],
        use_cases: "DX7-style sounds — bells, basses, electric pianos, evolving timbres.",
    },
    InstrumentMeta {
        source_type: SourceType::PhaseMod,
        name: "Phase Mod",
        short_name: "phasemod",
        category: "FM/Modulation",
        description: "Phase modulation synthesis",
        tags: &["digital", "bright", "evolving"],
        aliases: &["phase modulation"],
        use_cases: "Similar to FM but with different harmonic character. Bright, evolving tones.",
    },
    InstrumentMeta {
        source_type: SourceType::FMBell,
        name: "FM Bell",
        short_name: "fmbell",
        category: "FM/Modulation",
        description: "FM-tuned bell sound",
        tags: &["bell", "metallic", "bright", "percussive"],
        aliases: &["fm bell", "bell"],
        use_cases: "Bell tones, chimes, vibraphone-like sounds. Pre-tuned FM ratio for bells.",
    },
    InstrumentMeta {
        source_type: SourceType::FMBrass,
        name: "FM Brass",
        short_name: "fmbrass",
        category: "FM/Modulation",
        description: "FM-tuned brass sound",
        tags: &["brass", "bright", "punchy", "stab"],
        aliases: &["fm brass"],
        use_cases: "Brass stabs, bright punchy leads. Pre-tuned FM ratio for brass timbre.",
    },
    // --- Physical Models ---
    InstrumentMeta {
        source_type: SourceType::Pluck,
        name: "Pluck",
        short_name: "pluck",
        category: "Physical Models",
        description: "Karplus-Strong plucked string model",
        tags: &["pluck", "string", "realistic", "organic"],
        aliases: &["karplus-strong", "plucked string"],
        use_cases: "Realistic plucked strings, guitar-like tones, harpsichord textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Formant,
        name: "Formant",
        short_name: "formant",
        category: "Physical Models",
        description: "Vowel formant synthesizer",
        tags: &["vocal", "voice", "vowel", "organic"],
        aliases: &["vowel", "vocal"],
        use_cases: "Vocal-like sounds, talking synths. Morphs between vowel shapes.",
    },
    InstrumentMeta {
        source_type: SourceType::Bowed,
        name: "Bowed",
        short_name: "bowed",
        category: "Physical Models",
        description: "Bowed string physical model",
        tags: &["bowed", "string", "cello", "violin", "sustain"],
        aliases: &["bowed string", "violin", "cello"],
        use_cases: "Sustained string tones, cello/violin-like sounds, orchestral textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Blown,
        name: "Blown",
        short_name: "blown",
        category: "Physical Models",
        description: "Blown tube/reed physical model",
        tags: &["wind", "flute", "reed", "breathy", "organic"],
        aliases: &["flute", "reed", "wind"],
        use_cases: "Flute, clarinet, breathy tones. Physical model of air column vibration.",
    },
    InstrumentMeta {
        source_type: SourceType::Membrane,
        name: "Membrane",
        short_name: "membrane",
        category: "Physical Models",
        description: "Struck membrane physical model",
        tags: &["drum", "membrane", "percussive", "tuned"],
        aliases: &["drum membrane"],
        use_cases: "Tuned drums, timpani-like sounds, pitched percussion.",
    },
    // --- Mallet Percussion ---
    InstrumentMeta {
        source_type: SourceType::Marimba,
        name: "Marimba",
        short_name: "marimba",
        category: "Mallets",
        description: "Marimba mallet percussion",
        tags: &["mallet", "warm", "wooden", "percussive"],
        aliases: &[],
        use_cases: "Warm wooden mallet tones, melodic percussion, world music.",
    },
    InstrumentMeta {
        source_type: SourceType::Vibes,
        name: "Vibraphone",
        short_name: "vibes",
        category: "Mallets",
        description: "Vibraphone with motor vibrato",
        tags: &["mallet", "jazz", "warm", "metallic", "vibrato"],
        aliases: &["vibraphone", "vibe"],
        use_cases: "Jazz vibraphone, smooth melodic lines, dreamy textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Kalimba,
        name: "Kalimba",
        short_name: "kalimba",
        category: "Mallets",
        description: "Thumb piano / kalimba",
        tags: &["mallet", "pluck", "delicate", "african", "organic"],
        aliases: &["thumb piano", "mbira"],
        use_cases: "Delicate melodic patterns, African-inspired textures, lo-fi charm.",
    },
    InstrumentMeta {
        source_type: SourceType::SteelDrum,
        name: "Steel Drum",
        short_name: "steeldrum",
        category: "Mallets",
        description: "Caribbean steel drum (pan)",
        tags: &["mallet", "bright", "tropical", "metallic"],
        aliases: &["steel pan", "pan drum"],
        use_cases: "Tropical melodies, bright percussive leads, Caribbean feel.",
    },
    InstrumentMeta {
        source_type: SourceType::TubularBell,
        name: "Tubular Bell",
        short_name: "tubular",
        category: "Mallets",
        description: "Orchestral tubular bells",
        tags: &["bell", "orchestral", "bright", "sustain"],
        aliases: &["chime", "tubular bells"],
        use_cases: "Orchestral chimes, dramatic accents, cinematic bell hits.",
    },
    InstrumentMeta {
        source_type: SourceType::Glockenspiel,
        name: "Glockenspiel",
        short_name: "glock",
        category: "Mallets",
        description: "Bright metallic glockenspiel",
        tags: &["mallet", "bright", "sparkle", "high", "metallic"],
        aliases: &["glock", "bells"],
        use_cases: "Sparkling high melodies, music box textures, orchestral brightness.",
    },
    // --- Plucked Strings ---
    InstrumentMeta {
        source_type: SourceType::Guitar,
        name: "Guitar",
        short_name: "guitar",
        category: "Strings",
        description: "Acoustic guitar physical model",
        tags: &["string", "pluck", "acoustic", "warm", "organic"],
        aliases: &["acoustic guitar"],
        use_cases: "Acoustic guitar parts, strumming, fingerpicking textures.",
    },
    InstrumentMeta {
        source_type: SourceType::BassGuitar,
        name: "Bass Guitar",
        short_name: "bass",
        category: "Strings",
        description: "Electric bass guitar model",
        tags: &["bass", "pluck", "deep", "groove"],
        aliases: &["bass guitar", "electric bass"],
        use_cases: "Bass lines, groove foundations, slap bass textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Harp,
        name: "Harp",
        short_name: "harp",
        category: "Strings",
        description: "Orchestral harp model",
        tags: &["string", "pluck", "ethereal", "delicate"],
        aliases: &[],
        use_cases: "Glissandos, arpeggiated passages, ethereal ambient textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Koto,
        name: "Koto",
        short_name: "koto",
        category: "Strings",
        description: "Japanese koto string instrument",
        tags: &["string", "pluck", "japanese", "eastern", "organic"],
        aliases: &[],
        use_cases: "Japanese/Asian-inspired melodies, meditative textures.",
    },
    // --- Drums ---
    InstrumentMeta {
        source_type: SourceType::Kick,
        name: "Kick",
        short_name: "kick",
        category: "Drums",
        description: "Synthesized kick drum",
        tags: &["drum", "percussive", "bass", "punchy"],
        aliases: &["kick drum", "bass drum"],
        use_cases: "Foundation of rhythmic patterns, four-on-the-floor, boom-bap.",
    },
    InstrumentMeta {
        source_type: SourceType::Snare,
        name: "Snare",
        short_name: "snare",
        category: "Drums",
        description: "Synthesized snare drum",
        tags: &["drum", "percussive", "snappy", "crisp"],
        aliases: &["snare drum"],
        use_cases: "Backbeat, fills, rhythmic accents.",
    },
    InstrumentMeta {
        source_type: SourceType::HihatClosed,
        name: "Hi-Hat Closed",
        short_name: "hh_cl",
        category: "Drums",
        description: "Closed hi-hat",
        tags: &["drum", "percussive", "hi-hat", "tight"],
        aliases: &["closed hi-hat", "closed hat"],
        use_cases: "Rhythmic pulse, tight patterns, groove definition.",
    },
    InstrumentMeta {
        source_type: SourceType::HihatOpen,
        name: "Hi-Hat Open",
        short_name: "hh_op",
        category: "Drums",
        description: "Open hi-hat",
        tags: &["drum", "percussive", "hi-hat", "sustain"],
        aliases: &["open hi-hat", "open hat"],
        use_cases: "Accents, off-beat patterns, transition fills.",
    },
    InstrumentMeta {
        source_type: SourceType::Clap,
        name: "Clap",
        short_name: "clap",
        category: "Drums",
        description: "Synthesized hand clap",
        tags: &["drum", "percussive", "clap", "snappy"],
        aliases: &["hand clap"],
        use_cases: "Backbeat layer, house/techno patterns, rhythmic accents.",
    },
    InstrumentMeta {
        source_type: SourceType::Cowbell,
        name: "Cowbell",
        short_name: "cowbell",
        category: "Drums",
        description: "Synthesized cowbell",
        tags: &["drum", "percussive", "metallic", "latin"],
        aliases: &[],
        use_cases: "Latin rhythms, accent patterns, funk grooves.",
    },
    InstrumentMeta {
        source_type: SourceType::Rim,
        name: "Rim",
        short_name: "rim",
        category: "Drums",
        description: "Rim shot / cross-stick",
        tags: &["drum", "percussive", "sharp", "crisp"],
        aliases: &["rimshot", "cross-stick"],
        use_cases: "Ghost notes, hip-hop patterns, subtle rhythmic clicks.",
    },
    InstrumentMeta {
        source_type: SourceType::Tom,
        name: "Tom",
        short_name: "tom",
        category: "Drums",
        description: "Synthesized tom drum",
        tags: &["drum", "percussive", "deep", "fill"],
        aliases: &["tom drum"],
        use_cases: "Drum fills, tribal patterns, melodic percussion.",
    },
    InstrumentMeta {
        source_type: SourceType::Clave,
        name: "Clave",
        short_name: "clave",
        category: "Drums",
        description: "Wooden clave percussion",
        tags: &["drum", "percussive", "wooden", "latin", "click"],
        aliases: &[],
        use_cases: "Clave patterns, Latin rhythms, rhythmic skeleton.",
    },
    InstrumentMeta {
        source_type: SourceType::Conga,
        name: "Conga",
        short_name: "conga",
        category: "Drums",
        description: "Synthesized conga drum",
        tags: &["drum", "percussive", "latin", "deep", "hand"],
        aliases: &["conga drum"],
        use_cases: "Afro-Cuban rhythms, hand drum patterns, Latin percussion.",
    },
    // --- Classic Synths ---
    InstrumentMeta {
        source_type: SourceType::Choir,
        name: "Choir",
        short_name: "choir",
        category: "Classic Synths",
        description: "Synthesized choir pad",
        tags: &["pad", "vocal", "warm", "lush", "ethereal"],
        aliases: &["choir pad", "vocal pad"],
        use_cases: "Lush pads, ethereal backgrounds, cinematic atmosphere.",
    },
    InstrumentMeta {
        source_type: SourceType::EPiano,
        name: "Electric Piano",
        short_name: "epiano",
        category: "Classic Synths",
        description: "Rhodes-style electric piano",
        tags: &["keys", "warm", "jazz", "soul", "vintage"],
        aliases: &["rhodes", "electric piano", "e-piano", "wurlitzer"],
        use_cases: "Jazz chords, neo-soul, lo-fi hip-hop, warm melodic lines.",
    },
    InstrumentMeta {
        source_type: SourceType::Organ,
        name: "Organ",
        short_name: "organ",
        category: "Classic Synths",
        description: "Hammond-style organ",
        tags: &["keys", "vintage", "warm", "gospel", "rock"],
        aliases: &["hammond", "b3"],
        use_cases: "Gospel/soul chords, rock leads, vintage keyboard parts.",
    },
    InstrumentMeta {
        source_type: SourceType::BrassStab,
        name: "Brass Stab",
        short_name: "brass",
        category: "Classic Synths",
        description: "Punchy brass stab synth",
        tags: &["brass", "stab", "punchy", "bright", "disco"],
        aliases: &["brass", "stab"],
        use_cases: "Disco/funk stabs, horn hits, punchy melodic accents.",
    },
    InstrumentMeta {
        source_type: SourceType::Strings,
        name: "Strings",
        short_name: "strings",
        category: "Classic Synths",
        description: "Synthesized string ensemble",
        tags: &["strings", "pad", "lush", "orchestral", "warm"],
        aliases: &["string pad", "string ensemble"],
        use_cases: "Orchestral pads, cinematic swells, warm harmonic beds.",
    },
    InstrumentMeta {
        source_type: SourceType::Acid,
        name: "Acid",
        short_name: "acid",
        category: "Classic Synths",
        description: "303-style acid bass",
        tags: &["acid", "bass", "squelch", "resonant", "dance"],
        aliases: &["303", "tb-303", "acid bass"],
        use_cases: "Acid house bass lines, squelchy filter sweeps, techno.",
    },
    InstrumentMeta {
        source_type: SourceType::Universe,
        name: "Universe",
        short_name: "universe",
        category: "Classic Synths",
        description: "Cosmic pad synthesizer",
        tags: &["pad", "ambient", "cosmic", "wide", "evolving"],
        aliases: &["cosmic pad"],
        use_cases: "Ambient soundscapes, cosmic textures, deep evolving pads.",
    },
    InstrumentMeta {
        source_type: SourceType::Dreamscape,
        name: "Dreamscape",
        short_name: "dreamscape",
        category: "Classic Synths",
        description: "Dreamy texture synthesizer",
        tags: &["pad", "ambient", "dreamy", "ethereal", "soft"],
        aliases: &["dream pad"],
        use_cases: "Dreamy backgrounds, ambient beds, gentle evolving textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Soundtrack,
        name: "Soundtrack",
        short_name: "soundtrack",
        category: "Classic Synths",
        description: "Cinematic texture synthesizer",
        tags: &["pad", "cinematic", "dark", "tension", "evolving"],
        aliases: &["cinematic"],
        use_cases: "Film scoring, tension building, dramatic cinematic textures.",
    },
    // --- Experimental ---
    InstrumentMeta {
        source_type: SourceType::Gendy,
        name: "Gendy",
        short_name: "gendy",
        category: "Experimental",
        description: "Xenakis-style stochastic synthesis",
        tags: &[
            "experimental",
            "stochastic",
            "noise",
            "evolving",
            "avant-garde",
        ],
        aliases: &["stochastic", "xenakis"],
        use_cases: "Avant-garde textures, unpredictable evolving sounds, noise art.",
    },
    InstrumentMeta {
        source_type: SourceType::Chaos,
        name: "Chaos",
        short_name: "chaos",
        category: "Experimental",
        description: "Chaotic oscillator system",
        tags: &["experimental", "chaotic", "unpredictable", "noise"],
        aliases: &["chaotic"],
        use_cases: "Chaotic textures, glitchy sounds, experimental noise.",
    },
    // --- Synthesis ---
    InstrumentMeta {
        source_type: SourceType::Additive,
        name: "Additive",
        short_name: "additive",
        category: "Synthesis",
        description: "Additive harmonic synthesis",
        tags: &["additive", "harmonic", "precise", "organ-like"],
        aliases: &["additive synth"],
        use_cases: "Precise harmonic control, organ-like tones, spectral sculpting.",
    },
    InstrumentMeta {
        source_type: SourceType::Wavetable,
        name: "Wavetable",
        short_name: "wavetable",
        category: "Synthesis",
        description: "Wavetable oscillator with morphing",
        tags: &["wavetable", "digital", "morphing", "evolving"],
        aliases: &["wavetable synth"],
        use_cases: "Morphing timbres, modern digital sounds, evolving textures.",
    },
    InstrumentMeta {
        source_type: SourceType::Granular,
        name: "Granular",
        short_name: "granular",
        category: "Synthesis",
        description: "Granular synthesis engine",
        tags: &["granular", "texture", "ambient", "experimental"],
        aliases: &["granular synth"],
        use_cases: "Textural soundscapes, time-stretching, cloud-like ambient layers.",
    },
    // --- Routing ---
    InstrumentMeta {
        source_type: SourceType::AudioIn,
        name: "Audio In",
        short_name: "audio_in",
        category: "Routing",
        description: "Live audio input from sound card",
        tags: &["input", "live", "external"],
        aliases: &["audio input", "mic", "microphone"],
        use_cases: "Process live audio, guitar/mic input, external synth processing.",
    },
    InstrumentMeta {
        source_type: SourceType::BusIn,
        name: "Bus In",
        short_name: "bus_in",
        category: "Routing",
        description: "Receive audio from an internal bus",
        tags: &["input", "routing", "bus"],
        aliases: &["bus input"],
        use_cases: "Parallel processing, bus routing, effects returns.",
    },
    // --- Samplers ---
    InstrumentMeta {
        source_type: SourceType::PitchedSampler,
        name: "Pitched Sampler",
        short_name: "sample",
        category: "Samplers",
        description: "Pitch-tracking sample playback",
        tags: &["sampler", "sample", "pitched", "playback"],
        aliases: &["sampler", "pitched sampler"],
        use_cases: "Play samples across keyboard, pitched sample instruments.",
    },
    InstrumentMeta {
        source_type: SourceType::TimeStretch,
        name: "Time Stretch",
        short_name: "stretch",
        category: "Samplers",
        description: "Time-stretch sample playback",
        tags: &["sampler", "sample", "stretch", "granular"],
        aliases: &["time stretch", "timestretch"],
        use_cases: "Tempo-locked sample playback, time-independent pitch shifting.",
    },
    InstrumentMeta {
        source_type: SourceType::Kit,
        name: "Kit",
        short_name: "kit",
        category: "Samplers",
        description: "Multi-pad drum kit sampler",
        tags: &["sampler", "drum", "kit", "pads"],
        aliases: &["drum kit", "sample kit"],
        use_cases: "Sample-based drum kits, MPC-style pad triggering.",
    },
];

// ============================================================================
// Effect catalog
// ============================================================================

pub static EFFECT_CATALOG: &[EffectMeta] = &[
    // --- Time ---
    EffectMeta {
        effect_type: EffectType::Delay,
        name: "Delay",
        category: "Time",
        description: "Echo/delay with feedback",
        tags: &["time", "echo", "feedback", "space"],
        aliases: &["echo"],
        use_cases: "Rhythmic echoes, slapback, dub effects, spatial depth.",
    },
    EffectMeta {
        effect_type: EffectType::Reverb,
        name: "Reverb",
        category: "Time",
        description: "Algorithmic reverb",
        tags: &["time", "space", "room", "ambient"],
        aliases: &["verb", "room"],
        use_cases: "Spatial depth, room simulation, ambient washes, glue.",
    },
    EffectMeta {
        effect_type: EffectType::SpringReverb,
        name: "Spring Reverb",
        category: "Time",
        description: "Spring reverb emulation",
        tags: &["time", "spring", "vintage", "surf"],
        aliases: &["spring"],
        use_cases: "Surf guitar, vintage amp character, lo-fi warmth.",
    },
    // --- Dynamics ---
    EffectMeta {
        effect_type: EffectType::Gate,
        name: "Gate",
        category: "Dynamics",
        description: "Rhythmic amplitude gate",
        tags: &["dynamics", "rhythmic", "trance", "gate"],
        aliases: &["trance gate"],
        use_cases: "Rhythmic gating, sidechain-style pumping, stutter effects.",
    },
    EffectMeta {
        effect_type: EffectType::TapeComp,
        name: "Tape Comp",
        category: "Dynamics",
        description: "Tape-style compressor with warmth",
        tags: &["dynamics", "compression", "warm", "tape", "glue"],
        aliases: &["tape compressor", "compressor"],
        use_cases: "Warm compression, glue, tape saturation, dynamic control.",
    },
    EffectMeta {
        effect_type: EffectType::SidechainComp,
        name: "SC Comp",
        category: "Dynamics",
        description: "Sidechain compressor",
        tags: &["dynamics", "sidechain", "pump", "duck"],
        aliases: &["sidechain", "sidechain compressor", "sc comp"],
        use_cases: "Sidechain pumping, ducking, EDM rhythmic dynamics.",
    },
    EffectMeta {
        effect_type: EffectType::Limiter,
        name: "Limiter",
        category: "Dynamics",
        description: "Brickwall limiter",
        tags: &["dynamics", "limiter", "loudness", "mastering"],
        aliases: &[],
        use_cases: "Peak limiting, loudness maximizing, mastering chain.",
    },
    EffectMeta {
        effect_type: EffectType::MultibandComp,
        name: "MB Comp",
        category: "Dynamics",
        description: "Multiband compressor",
        tags: &["dynamics", "multiband", "mastering", "precision"],
        aliases: &["multiband compressor", "multiband comp"],
        use_cases: "Frequency-selective dynamics, mastering, taming harsh bands.",
    },
    // --- Modulation ---
    EffectMeta {
        effect_type: EffectType::Chorus,
        name: "Chorus",
        category: "Modulation",
        description: "Chorus effect",
        tags: &["modulation", "chorus", "width", "lush"],
        aliases: &[],
        use_cases: "Thickening, stereo width, lush pads, 80s character.",
    },
    EffectMeta {
        effect_type: EffectType::Flanger,
        name: "Flanger",
        category: "Modulation",
        description: "Flanging effect",
        tags: &["modulation", "flanger", "jet", "metallic"],
        aliases: &[],
        use_cases: "Jet sweep, metallic textures, psychedelic effects.",
    },
    EffectMeta {
        effect_type: EffectType::Phaser,
        name: "Phaser",
        category: "Modulation",
        description: "Phaser effect",
        tags: &["modulation", "phaser", "sweep", "vintage"],
        aliases: &[],
        use_cases: "Sweeping phase, vintage character, funk rhythm guitar.",
    },
    EffectMeta {
        effect_type: EffectType::Tremolo,
        name: "Tremolo",
        category: "Modulation",
        description: "Amplitude tremolo",
        tags: &["modulation", "tremolo", "rhythmic", "vintage"],
        aliases: &["trem"],
        use_cases: "Rhythmic amplitude variation, vintage amp character.",
    },
    EffectMeta {
        effect_type: EffectType::Autopan,
        name: "Autopan",
        category: "Modulation",
        description: "Automatic stereo panning",
        tags: &["modulation", "pan", "stereo", "movement"],
        aliases: &["auto pan", "auto panner"],
        use_cases: "Stereo movement, spatial animation, psychedelic panning.",
    },
    EffectMeta {
        effect_type: EffectType::Leslie,
        name: "Leslie",
        category: "Modulation",
        description: "Leslie rotary speaker emulation",
        tags: &["modulation", "rotary", "vintage", "organ"],
        aliases: &["rotary", "leslie speaker"],
        use_cases: "Organ processing, vintage rotary character, warm modulation.",
    },
    // --- Distortion ---
    EffectMeta {
        effect_type: EffectType::Distortion,
        name: "Distortion",
        category: "Distortion",
        description: "Waveshaping distortion",
        tags: &["distortion", "drive", "dirty", "aggressive"],
        aliases: &["overdrive", "drive"],
        use_cases: "Adding grit, aggressive character, guitar-amp-like drive.",
    },
    EffectMeta {
        effect_type: EffectType::Bitcrusher,
        name: "Bitcrusher",
        category: "Distortion",
        description: "Bit depth and sample rate reduction",
        tags: &["distortion", "lofi", "retro", "digital", "8bit"],
        aliases: &["bit crusher", "lo-fi"],
        use_cases: "Lo-fi aesthetics, retro game sounds, digital degradation.",
    },
    EffectMeta {
        effect_type: EffectType::Wavefolder,
        name: "Wavefolder",
        category: "Distortion",
        description: "Waveshaping via wavefolding",
        tags: &["distortion", "wavefold", "metallic", "complex"],
        aliases: &["wave folder"],
        use_cases: "Complex harmonics, metallic textures, Buchla-style timbres.",
    },
    EffectMeta {
        effect_type: EffectType::Saturator,
        name: "Saturator",
        category: "Distortion",
        description: "Soft-clip saturation",
        tags: &["distortion", "saturation", "warm", "analog"],
        aliases: &["soft clip"],
        use_cases: "Gentle warmth, analog character, subtle harmonic enhancement.",
    },
    // --- EQ ---
    EffectMeta {
        effect_type: EffectType::TiltEq,
        name: "Tilt EQ",
        category: "EQ",
        description: "Single-knob tilt equalizer",
        tags: &["eq", "tilt", "simple", "tone"],
        aliases: &["tilt eq", "tilt"],
        use_cases: "Quick tonal balance, bright/dark adjustment, simple tone shaping.",
    },
    EffectMeta {
        effect_type: EffectType::ParaEq,
        name: "Para EQ",
        category: "EQ",
        description: "Parametric equalizer",
        tags: &["eq", "parametric", "surgical", "precise"],
        aliases: &["parametric eq", "parametric"],
        use_cases: "Surgical frequency shaping, notch filtering, precise tonal control.",
    },
    // --- Stereo ---
    EffectMeta {
        effect_type: EffectType::StereoWidener,
        name: "Stereo Widener",
        category: "Stereo",
        description: "Stereo image widener",
        tags: &["stereo", "width", "space", "imaging"],
        aliases: &["widener", "stereo width"],
        use_cases: "Widening stereo image, creating space, master bus enhancement.",
    },
    EffectMeta {
        effect_type: EffectType::MidSide,
        name: "Mid/Side",
        category: "Stereo",
        description: "Mid/side processing",
        tags: &["stereo", "midside", "imaging", "mastering"],
        aliases: &["mid side", "ms"],
        use_cases: "Independent mid/side control, mastering, stereo sculpting.",
    },
    EffectMeta {
        effect_type: EffectType::Crossfader,
        name: "Crossfader",
        category: "Stereo",
        description: "DJ-style crossfader",
        tags: &["stereo", "crossfade", "dj", "mix"],
        aliases: &["xfader"],
        use_cases: "DJ transitions, stereo balance control, A/B mixing.",
    },
    // --- Pitch ---
    EffectMeta {
        effect_type: EffectType::PitchShifter,
        name: "Pitch Shifter",
        category: "Pitch",
        description: "Real-time pitch shifting",
        tags: &["pitch", "shift", "harmony", "transpose"],
        aliases: &["pitch shift", "harmonizer"],
        use_cases: "Harmonies, octave effects, pitch correction, creative detuning.",
    },
    EffectMeta {
        effect_type: EffectType::Autotune,
        name: "Autotune",
        category: "Pitch",
        description: "Automatic pitch correction",
        tags: &["pitch", "correction", "vocal", "tuning"],
        aliases: &["pitch correction", "auto-tune"],
        use_cases: "Vocal pitch correction, T-Pain effect, subtle intonation fix.",
    },
    EffectMeta {
        effect_type: EffectType::FreqShifter,
        name: "Freq Shifter",
        category: "Pitch",
        description: "Frequency shifting (not pitch shifting)",
        tags: &["pitch", "frequency", "shift", "metallic", "experimental"],
        aliases: &["frequency shifter"],
        use_cases: "Metallic textures, Bode-style shifting, experimental sound design.",
    },
    // --- Granular ---
    EffectMeta {
        effect_type: EffectType::GranularDelay,
        name: "Granular Delay",
        category: "Granular",
        description: "Granular processing with delay",
        tags: &["granular", "delay", "texture", "ambient"],
        aliases: &["grain delay"],
        use_cases: "Textural echoes, ambient washes, granular clouds.",
    },
    EffectMeta {
        effect_type: EffectType::GranularFreeze,
        name: "Granular Freeze",
        category: "Granular",
        description: "Granular freeze/sustain effect",
        tags: &["granular", "freeze", "sustain", "pad"],
        aliases: &["freeze"],
        use_cases: "Freezing moments in time, infinite sustain, drone generation.",
    },
    // --- Spectral ---
    EffectMeta {
        effect_type: EffectType::SpectralFreeze,
        name: "Spectral Freeze",
        category: "Spectral",
        description: "FFT spectral freeze",
        tags: &["spectral", "freeze", "fft", "ambient"],
        aliases: &["spectral hold"],
        use_cases: "Spectral sustain, frozen harmonics, ambient sound design.",
    },
    EffectMeta {
        effect_type: EffectType::Glitch,
        name: "Glitch",
        category: "Spectral",
        description: "Buffer-based glitch effects",
        tags: &["spectral", "glitch", "stutter", "experimental"],
        aliases: &["stutter"],
        use_cases: "Glitch art, buffer manipulation, stutter edits, IDM.",
    },
    EffectMeta {
        effect_type: EffectType::Denoise,
        name: "Denoise",
        category: "Spectral",
        description: "Spectral noise reduction",
        tags: &["spectral", "denoise", "clean", "noise reduction"],
        aliases: &["noise reduction"],
        use_cases: "Cleaning recordings, reducing noise floor, audio restoration.",
    },
    // --- Convolution ---
    EffectMeta {
        effect_type: EffectType::ConvolutionReverb,
        name: "Conv Reverb",
        category: "Convolution",
        description: "Convolution reverb with impulse responses",
        tags: &["reverb", "convolution", "realistic", "space"],
        aliases: &["convolution reverb", "ir reverb", "impulse reverb"],
        use_cases: "Realistic spaces, sampled room acoustics, cabinet simulation.",
    },
    // --- Character ---
    EffectMeta {
        effect_type: EffectType::Vinyl,
        name: "Vinyl",
        category: "Character",
        description: "Vinyl record emulation",
        tags: &["lofi", "vinyl", "crackle", "warm", "vintage"],
        aliases: &["vinyl emulation", "record"],
        use_cases: "Lo-fi warmth, vinyl crackle/noise, vintage character.",
    },
    EffectMeta {
        effect_type: EffectType::Cabinet,
        name: "Cabinet",
        category: "Character",
        description: "Speaker cabinet emulation",
        tags: &["cabinet", "amp", "speaker", "character"],
        aliases: &["cab", "speaker sim"],
        use_cases: "Guitar/bass amp sim, speaker coloring, warmth and presence.",
    },
    // --- Synthesis ---
    EffectMeta {
        effect_type: EffectType::RingMod,
        name: "Ring Mod",
        category: "Synthesis",
        description: "Ring modulation effect",
        tags: &["synthesis", "ring", "metallic", "inharmonic"],
        aliases: &["ring modulation"],
        use_cases: "Metallic textures, robotic voices, bell-like inharmonic tones.",
    },
    EffectMeta {
        effect_type: EffectType::Resonator,
        name: "Resonator",
        category: "Synthesis",
        description: "Tuned resonator bank",
        tags: &["synthesis", "resonance", "tuned", "harmonic"],
        aliases: &[],
        use_cases: "Adding pitched resonance, sympathetic vibrations, tonal coloring.",
    },
    EffectMeta {
        effect_type: EffectType::Vocoder,
        name: "Vocoder",
        category: "Synthesis",
        description: "Channel vocoder",
        tags: &["synthesis", "vocoder", "voice", "robotic"],
        aliases: &[],
        use_cases: "Robot voice, vocal synthesis, talkbox-like effects.",
    },
    // --- Envelope ---
    EffectMeta {
        effect_type: EffectType::EnvFollower,
        name: "Env Follower",
        category: "Envelope",
        description: "Envelope follower with filter",
        tags: &["envelope", "follower", "dynamic", "filter"],
        aliases: &["envelope follower"],
        use_cases: "Auto-wah, dynamic filtering, amplitude-driven effects.",
    },
    EffectMeta {
        effect_type: EffectType::WahPedal,
        name: "Wah Pedal",
        category: "Envelope",
        description: "Wah-wah pedal emulation",
        tags: &["envelope", "wah", "guitar", "funk"],
        aliases: &["wah", "wah-wah"],
        use_cases: "Funk guitar, expressive sweeps, cocked wah tone.",
    },
];

// ============================================================================
// Filter catalog
// ============================================================================

pub static FILTER_CATALOG: &[FilterMeta] = &[
    FilterMeta {
        filter_type: FilterType::Lpf,
        name: "Low-Pass",
        description: "Passes frequencies below cutoff",
        tags: &["subtractive", "warm", "dark"],
    },
    FilterMeta {
        filter_type: FilterType::Hpf,
        name: "High-Pass",
        description: "Passes frequencies above cutoff",
        tags: &["thin", "bright", "clean"],
    },
    FilterMeta {
        filter_type: FilterType::Bpf,
        name: "Band-Pass",
        description: "Passes a band of frequencies around cutoff",
        tags: &["nasal", "focused", "telephone"],
    },
    FilterMeta {
        filter_type: FilterType::Notch,
        name: "Notch",
        description: "Removes a narrow band of frequencies",
        tags: &["cut", "notch", "surgical"],
    },
    FilterMeta {
        filter_type: FilterType::Comb,
        name: "Comb",
        description: "Comb filter with pitched resonance",
        tags: &["resonant", "metallic", "tuned"],
    },
    FilterMeta {
        filter_type: FilterType::Allpass,
        name: "Allpass",
        description: "Phase-shifting allpass filter",
        tags: &["phase", "diffusion", "subtle"],
    },
    FilterMeta {
        filter_type: FilterType::Vowel,
        name: "Vowel",
        description: "Vowel formant filter",
        tags: &["vocal", "formant", "character"],
    },
    FilterMeta {
        filter_type: FilterType::ResDrive,
        name: "ResDrive",
        description: "Resonant filter with drive/saturation",
        tags: &["resonant", "drive", "aggressive"],
    },
    FilterMeta {
        filter_type: FilterType::ParametricEq,
        name: "Parametric",
        description: "Parametric EQ as filter stage",
        tags: &["eq", "precise", "surgical"],
    },
];

// ============================================================================
// Lookup methods on SourceType
// ============================================================================

impl SourceType {
    /// Get the metadata struct for this source type.
    /// Returns None for Custom/Vst variants (not in static catalog).
    pub fn meta(&self) -> Option<&'static InstrumentMeta> {
        match self {
            SourceType::Custom(_) | SourceType::Vst(_) => None,
            _ => INSTRUMENT_CATALOG.iter().find(|m| m.source_type == *self),
        }
    }

    /// Fuzzy lookup from natural language — checks name, short_name, and aliases (case-insensitive).
    pub fn from_natural_language(input: &str) -> Option<SourceType> {
        let lower = input.to_lowercase();
        INSTRUMENT_CATALOG.iter().find_map(|m| {
            if m.name.to_lowercase() == lower
                || m.short_name.to_lowercase() == lower
                || m.aliases.iter().any(|a| a.to_lowercase() == lower)
            {
                Some(m.source_type)
            } else {
                None
            }
        })
    }
}

// ============================================================================
// Lookup methods on EffectType
// ============================================================================

impl EffectType {
    /// Get the metadata struct for this effect type.
    /// Returns None for Vst variants (not in static catalog).
    pub fn meta(&self) -> Option<&'static EffectMeta> {
        match self {
            EffectType::Vst(_) => None,
            _ => EFFECT_CATALOG.iter().find(|m| m.effect_type == *self),
        }
    }

    /// Fuzzy lookup from natural language — checks name and aliases (case-insensitive).
    pub fn from_natural_language(input: &str) -> Option<EffectType> {
        let lower = input.to_lowercase();
        EFFECT_CATALOG.iter().find_map(|m| {
            if m.name.to_lowercase() == lower || m.aliases.iter().any(|a| a.to_lowercase() == lower)
            {
                Some(m.effect_type)
            } else {
                None
            }
        })
    }
}

// ============================================================================
// Lookup methods on FilterType
// ============================================================================

impl FilterType {
    /// Get the metadata struct for this filter type.
    pub fn meta(&self) -> &'static FilterMeta {
        FILTER_CATALOG
            .iter()
            .find(|m| m.filter_type == *self)
            .expect("every FilterType variant must have a catalog entry")
    }
}

// ============================================================================
// Catalog query functions
// ============================================================================

/// Find instruments by tag (e.g. "warm", "percussive").
pub fn instruments_by_tag(tag: &str) -> Vec<&'static InstrumentMeta> {
    let lower = tag.to_lowercase();
    INSTRUMENT_CATALOG
        .iter()
        .filter(|m| m.tags.iter().any(|t| t.to_lowercase() == lower))
        .collect()
}

/// Find instruments by category (e.g. "Physical Models").
pub fn instruments_by_category(category: &str) -> Vec<&'static InstrumentMeta> {
    let lower = category.to_lowercase();
    INSTRUMENT_CATALOG
        .iter()
        .filter(|m| m.category.to_lowercase() == lower)
        .collect()
}

/// All unique instrument categories.
pub fn instrument_categories() -> Vec<&'static str> {
    let mut cats: Vec<&'static str> = INSTRUMENT_CATALOG.iter().map(|m| m.category).collect();
    cats.dedup(); // Catalog is ordered by category, so dedup suffices
    cats
}

/// Find effects by tag (e.g. "modulation", "warm").
pub fn effects_by_tag(tag: &str) -> Vec<&'static EffectMeta> {
    let lower = tag.to_lowercase();
    EFFECT_CATALOG
        .iter()
        .filter(|m| m.tags.iter().any(|t| t.to_lowercase() == lower))
        .collect()
}

/// Find effects by category (e.g. "Dynamics", "Modulation").
pub fn effects_by_category(category: &str) -> Vec<&'static EffectMeta> {
    let lower = category.to_lowercase();
    EFFECT_CATALOG
        .iter()
        .filter(|m| m.category.to_lowercase() == lower)
        .collect()
}

/// All unique effect categories.
pub fn effect_categories() -> Vec<&'static str> {
    let mut cats: Vec<&'static str> = EFFECT_CATALOG.iter().map(|m| m.category).collect();
    cats.dedup();
    cats
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_builtin_source_type_has_catalog_entry() {
        for st in SourceType::all() {
            assert!(
                st.meta().is_some(),
                "SourceType::{:?} has no catalog entry",
                st
            );
        }
    }

    #[test]
    fn every_builtin_effect_type_has_catalog_entry() {
        for et in EffectType::all() {
            assert!(
                et.meta().is_some(),
                "EffectType::{:?} has no catalog entry",
                et
            );
        }
    }

    #[test]
    fn every_filter_type_has_catalog_entry() {
        for ft in FilterType::all() {
            // meta() panics if missing, so just call it
            let _ = ft.meta();
        }
    }

    #[test]
    fn catalog_entries_have_non_empty_fields() {
        for m in INSTRUMENT_CATALOG {
            assert!(!m.name.is_empty(), "{:?} name empty", m.source_type);
            assert!(
                !m.short_name.is_empty(),
                "{:?} short_name empty",
                m.source_type
            );
            assert!(!m.category.is_empty(), "{:?} category empty", m.source_type);
            assert!(
                !m.description.is_empty(),
                "{:?} description empty",
                m.source_type
            );
            assert!(
                !m.use_cases.is_empty(),
                "{:?} use_cases empty",
                m.source_type
            );
        }
        for m in EFFECT_CATALOG {
            assert!(!m.name.is_empty(), "{:?} name empty", m.effect_type);
            assert!(!m.category.is_empty(), "{:?} category empty", m.effect_type);
            assert!(
                !m.description.is_empty(),
                "{:?} description empty",
                m.effect_type
            );
            assert!(
                !m.use_cases.is_empty(),
                "{:?} use_cases empty",
                m.effect_type
            );
        }
        for m in FILTER_CATALOG {
            assert!(!m.name.is_empty(), "{:?} name empty", m.filter_type);
            assert!(
                !m.description.is_empty(),
                "{:?} description empty",
                m.filter_type
            );
        }
    }

    #[test]
    fn instrument_catalog_count_matches_all() {
        assert_eq!(
            INSTRUMENT_CATALOG.len(),
            SourceType::all().len(),
            "catalog entries should match SourceType::all() count"
        );
    }

    #[test]
    fn effect_catalog_count_matches_all() {
        assert_eq!(
            EFFECT_CATALOG.len(),
            EffectType::all().len(),
            "catalog entries should match EffectType::all() count"
        );
    }

    #[test]
    fn filter_catalog_count_matches_all() {
        assert_eq!(
            FILTER_CATALOG.len(),
            FilterType::all().len(),
            "catalog entries should match FilterType::all() count"
        );
    }

    #[test]
    fn from_natural_language_exact_name() {
        assert_eq!(
            SourceType::from_natural_language("Saw"),
            Some(SourceType::Saw)
        );
        assert_eq!(
            SourceType::from_natural_language("Electric Piano"),
            Some(SourceType::EPiano)
        );
        assert_eq!(
            SourceType::from_natural_language("Kick"),
            Some(SourceType::Kick)
        );
    }

    #[test]
    fn from_natural_language_short_name() {
        assert_eq!(
            SourceType::from_natural_language("saw"),
            Some(SourceType::Saw)
        );
        assert_eq!(
            SourceType::from_natural_language("epiano"),
            Some(SourceType::EPiano)
        );
        assert_eq!(
            SourceType::from_natural_language("hh_cl"),
            Some(SourceType::HihatClosed)
        );
    }

    #[test]
    fn from_natural_language_alias() {
        assert_eq!(
            SourceType::from_natural_language("rhodes"),
            Some(SourceType::EPiano)
        );
        assert_eq!(
            SourceType::from_natural_language("sawtooth"),
            Some(SourceType::Saw)
        );
        assert_eq!(
            SourceType::from_natural_language("303"),
            Some(SourceType::Acid)
        );
        assert_eq!(
            SourceType::from_natural_language("kick drum"),
            Some(SourceType::Kick)
        );
    }

    #[test]
    fn from_natural_language_case_insensitive() {
        assert_eq!(
            SourceType::from_natural_language("SAW"),
            Some(SourceType::Saw)
        );
        assert_eq!(
            SourceType::from_natural_language("Rhodes"),
            Some(SourceType::EPiano)
        );
        assert_eq!(
            SourceType::from_natural_language("NOISE"),
            Some(SourceType::Noise)
        );
    }

    #[test]
    fn from_natural_language_unknown_returns_none() {
        assert_eq!(SourceType::from_natural_language("nonexistent"), None);
        assert_eq!(SourceType::from_natural_language(""), None);
    }

    #[test]
    fn effect_from_natural_language_exact_name() {
        assert_eq!(
            EffectType::from_natural_language("Delay"),
            Some(EffectType::Delay)
        );
        assert_eq!(
            EffectType::from_natural_language("Reverb"),
            Some(EffectType::Reverb)
        );
    }

    #[test]
    fn effect_from_natural_language_alias() {
        assert_eq!(
            EffectType::from_natural_language("echo"),
            Some(EffectType::Delay)
        );
        assert_eq!(
            EffectType::from_natural_language("compressor"),
            Some(EffectType::TapeComp)
        );
        assert_eq!(
            EffectType::from_natural_language("sidechain"),
            Some(EffectType::SidechainComp)
        );
    }

    #[test]
    fn effect_from_natural_language_case_insensitive() {
        assert_eq!(
            EffectType::from_natural_language("REVERB"),
            Some(EffectType::Reverb)
        );
        assert_eq!(
            EffectType::from_natural_language("chorus"),
            Some(EffectType::Chorus)
        );
    }

    #[test]
    fn instruments_by_tag_returns_expected() {
        let warm = instruments_by_tag("warm");
        assert!(!warm.is_empty());
        // Triangle is tagged "warm"
        assert!(warm.iter().any(|m| m.source_type == SourceType::Tri));
    }

    #[test]
    fn instruments_by_category_returns_expected() {
        let drums = instruments_by_category("Drums");
        assert!(!drums.is_empty());
        assert!(drums.iter().any(|m| m.source_type == SourceType::Kick));
        assert!(drums.iter().any(|m| m.source_type == SourceType::Snare));
    }

    #[test]
    fn instruments_by_category_case_insensitive() {
        let drums = instruments_by_category("drums");
        assert!(!drums.is_empty());
    }

    #[test]
    fn instrument_categories_non_empty() {
        let cats = instrument_categories();
        assert!(!cats.is_empty());
        assert!(cats.contains(&"Oscillators"));
        assert!(cats.contains(&"Drums"));
    }

    #[test]
    fn effects_by_tag_returns_expected() {
        let modulation = effects_by_tag("modulation");
        assert!(!modulation.is_empty());
        assert!(modulation
            .iter()
            .any(|m| m.effect_type == EffectType::Chorus));
    }

    #[test]
    fn effects_by_category_returns_expected() {
        let dynamics = effects_by_category("Dynamics");
        assert!(!dynamics.is_empty());
        assert!(dynamics
            .iter()
            .any(|m| m.effect_type == EffectType::Limiter));
    }

    #[test]
    fn effect_categories_non_empty() {
        let cats = effect_categories();
        assert!(!cats.is_empty());
        assert!(cats.contains(&"Time"));
        assert!(cats.contains(&"Dynamics"));
    }

    #[test]
    fn custom_and_vst_source_types_return_none_meta() {
        use crate::CustomSynthDefId;
        assert!(SourceType::Custom(CustomSynthDefId::new(1))
            .meta()
            .is_none());
    }

    #[test]
    fn vst_effect_returns_none_meta() {
        use crate::VstPluginId;
        assert!(EffectType::Vst(VstPluginId::new(1)).meta().is_none());
    }

    #[test]
    fn instrument_catalog_serializable() {
        let json = serde_json::to_string(INSTRUMENT_CATALOG).unwrap();
        assert!(!json.is_empty());
    }

    #[test]
    fn effect_catalog_serializable() {
        let json = serde_json::to_string(EFFECT_CATALOG).unwrap();
        assert!(!json.is_empty());
    }

    #[test]
    fn filter_catalog_serializable() {
        let json = serde_json::to_string(FILTER_CATALOG).unwrap();
        assert!(!json.is_empty());
    }
}
