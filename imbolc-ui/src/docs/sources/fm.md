# FM Synthesis

Modulation-oriented synthesis sources.

Most entries share these baseline controls:
- `freq`: base frequency in hertz.
- `amp`: source output level.
- `lag`: smoothing applied to control changes.
- `attack`, `decay`, `sustain`, `release`: source envelope.

The sections below list source-specific controls.

## FM

Two-operator frequency modulation source.

Source-specific controls:
- `ratio`: modulator-to-carrier frequency ratio.
- `index`: modulation depth.

## Ring Mod

Ring modulation source using a controllable modulator ratio and depth.

Source-specific controls:
- `mod_ratio`: modulation frequency ratio.
- `mod_depth`: modulation amount.

## Feedback Sine

Sine oscillator with self-feedback.

Source-specific controls:
- `feedback`: feedback amount.

## Phase Mod

Phase modulation source with FM-like spectral behavior.

Source-specific controls:
- `ratio`: modulator ratio.
- `index`: phase modulation depth.

## FM Bell

Bell-oriented FM-derived preset voice.

Source-specific controls:
- `brightness`: high partial emphasis.

## FM Brass

Brass-oriented FM-derived preset voice.

Source-specific controls:
- `brightness`: harmonic edge/brightness.
