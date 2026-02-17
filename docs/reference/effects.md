# Effects Reference

All 39 built-in effect types and 9 filter types available in Imbolc.

Add effects with `a` in the Track Editor (F1, then Enter on a track) or in the Mixer detail view (F4, then Enter on a channel). Toggle effect bypass with `b`.

---

## Time-Based (3)

| Name | Description |
|------|-------------|
| Delay | Feedback delay with time and mix controls |
| Reverb | Algorithmic reverb with room size and damping |
| Spring Reverb | Spring reverb emulation |

## Dynamics (5)

| Name | Description |
|------|-------------|
| Gate | Rhythmic amplitude gate |
| Tape Comp | Tape-style compressor with drive and saturation |
| SC Comp | Sidechain compressor (keyed from a bus) |
| Limiter | Brick-wall limiter |
| MB Comp | Multiband compressor (3-band) |

## Modulation (6)

| Name | Description |
|------|-------------|
| Chorus | Chorus with rate and depth |
| Flanger | Flanger with feedback |
| Phaser | Multi-stage phaser |
| Tremolo | Amplitude tremolo with shape control |
| Autopan | Automatic stereo panning |
| Leslie | Rotary speaker emulation |

## Distortion (4)

| Name | Description |
|------|-------------|
| Distortion | Distortion with drive, mode, and tone |
| Bitcrusher | Sample rate and bit depth reduction |
| Wavefolder | Wavefolding distortion |
| Saturator | Soft saturation with color control |

## EQ (2)

| Name | Description |
|------|-------------|
| Tilt EQ | Single-knob tilt equalizer |
| Para EQ | 3-band parametric EQ |

## Stereo (3)

| Name | Description |
|------|-------------|
| Stereo Widener | Stereo width adjustment |
| Mid/Side | Mid/side processing with independent gain |
| Crossfader | Crossfade between dry signal and a bus |

## Pitch (3)

| Name | Description |
|------|-------------|
| Pitch Shifter | Pitch shifting in semitones |
| Autotune | Pitch correction |
| Freq Shifter | Frequency shifting in Hz |

## Granular (2)

| Name | Description |
|------|-------------|
| Granular Delay | Granular delay with pitch and density |
| Granular Freeze | Granular freeze/sustain effect |

## Spectral (3)

| Name | Description |
|------|-------------|
| Spectral Freeze | FFT-based spectral freeze |
| Glitch | Buffer-based glitch effect |
| Denoise | Spectral noise reduction |

## Convolution (1)

| Name | Description |
|------|-------------|
| Conv Reverb | Convolution reverb with impulse response |

## Character (2)

| Name | Description |
|------|-------------|
| Vinyl | Vinyl record emulation (wow, flutter, noise, hiss) |
| Cabinet | Speaker cabinet simulation |

## Synthesis (3)

| Name | Description |
|------|-------------|
| Ring Mod | Ring modulation effect |
| Resonator | Tuned resonator |
| Vocoder | Multi-band vocoder |

## Envelope (2)

| Name | Description |
|------|-------------|
| Env Follower | Envelope follower with attack/release |
| Wah Pedal | Auto-wah / envelope-controlled filter |

## External (1)

| Name | Description |
|------|-------------|
| VST | VST effect via VSTPlugin |

---

## Filter Types

Each track has an optional filter in its processing chain. Toggle with `f` in the Track Editor; cycle type with `t`. All filters have cutoff and resonance parameters.

| Name | Description |
|------|-------------|
| Low-Pass | Passes frequencies below cutoff |
| High-Pass | Passes frequencies above cutoff |
| Band-Pass | Passes a band around cutoff frequency |
| Notch | Rejects a band around cutoff frequency |
| Comb | Comb filter (creates harmonic resonances) |
| Allpass | Phase-shifting allpass filter |
| Vowel | Formant / vowel filter with shape control |
| ResDrive | Resonant filter with drive |
| Parametric | Parametric EQ band with frequency, Q, and gain |
