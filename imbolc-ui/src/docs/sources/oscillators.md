# Oscillators

Oscillator sources generate sound directly (without sample playback or external
audio input).

Most pitched entries on this page share these baseline controls:
- `freq`: base frequency in hertz.
- `amp`: source output level.
- `lag`: smoothing applied to control changes.
- `attack`, `decay`, `sustain`, `release`: source envelope.

The sections below focus on source-specific controls.

## Saw

Sawtooth oscillator with a full harmonic series.

Source-specific controls:
- None.

## Sine

Sine oscillator containing only the fundamental.

Source-specific controls:
- None.

## Square

Square-like pulse oscillator centered at a fixed 50% duty cycle.

Source-specific controls:
- None (pulse width is fixed unless modulated externally).

## Triangle

Triangle oscillator with reduced high-frequency harmonic energy.

Source-specific controls:
- None.

## Noise

Noise source with selectable coloration and event density.

Source-specific controls:
- `color`: selects the noise coloration mode.
- `density`: controls event density in sparse/noise modes.

## Pulse

Variable-duty pulse oscillator.

Source-specific controls:
- `width`: duty cycle.

## SuperSaw

Detuned multi-voice saw oscillator.

Source-specific controls:
- `detune`: spread between voices.
- `mix`: blend of center and detuned components.

## Sync

Hard-sync oscillator where a slave oscillator is reset by a master period.

Source-specific controls:
- `sync_ratio`: slave-to-master frequency ratio.

## Choir

Vocal-formant style synth voice.

Source-specific controls:
- `vowel`: formant position.
- `spread`: stereo/unison spread.
- `vibrato`: pitch modulation depth.

## EPiano

Electric-piano style voice.

Source-specific controls:
- `tine`: high partial emphasis.
- `bark`: attack nonlinearity.
- `tremolo`: amplitude modulation depth.

## Organ

Drawbar organ model.

Source-specific controls:
- `perc`: key-click/percussion amount.
- `leslie`: rotary speaker modulation amount.
- `d1` through `d9`: drawbar harmonic levels.

## Brass Stab

Brass-inspired synth stab.

Source-specific controls:
- `bite`: transient brightness and edge.
- `cutoff_decay`: filter-envelope decay amount.

## Strings

Ensemble string-style synth voice.

Source-specific controls:
- `spread`: detune/unison spread.
- `vibrato`: pitch modulation depth.

## Acid

Resonant monosynth bass voice.

Source-specific controls:
- `cutoff`: filter cutoff frequency.
- `res`: filter resonance.
- `accent`: accent amount.
- `envmod`: envelope-to-filter depth.

## Universe

Layered pad voice designed for long, evolving tones.

Source-specific controls:
- `shimmer`: high-frequency sheen.
- `warmth`: low-mid body.
- `air`: high-band openness.

## Dreamscape

Atmospheric pad voice with slow spectral movement.

Source-specific controls:
- `density`: layer thickness.
- `evolve`: modulation/evolution amount.
- `brightness`: high-frequency emphasis.

## Soundtrack

Cinematic synth texture voice.

Source-specific controls:
- `brass`: brass-like component level.
- `sweep`: spectral movement amount.
- `width`: stereo spread.

## Additive

Additive oscillator built from harmonic partials.

Source-specific controls:
- `harmonics`: number of active partials.
- `rolloff`: amplitude decay across partial index.

## Wavetable

Wavetable oscillator with continuous table scan.

Source-specific controls:
- `position`: scan position in the wavetable.

## Gendy

Stochastic waveform generator (dynamic stochastic synthesis).

Source-specific controls:
- `ampdist`: amplitude-distribution mode.
- `durdist`: duration-distribution mode.
- `minfreq`: lower frequency bound.
- `maxfreq`: upper frequency bound.

## Chaos

Chaotic-system oscillator.

Source-specific controls:
- `model`: chaotic model selector.
- `chaos_freq`: update/audio rate of the chaotic system.
- `chaos_param`: model control parameter.
