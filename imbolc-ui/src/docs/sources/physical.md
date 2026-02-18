# Physical Models

Physically inspired and resonator-based synthesis sources.

Most entries share these baseline controls:
- `freq`: base frequency in hertz.
- `amp`: source output level.
- `lag`: smoothing applied to control changes.
- `attack`, `decay`, `sustain`, `release`: source envelope.

The sections below list model-specific controls.

## Pluck

Karplus-Strong style plucked-string model.

Source-specific controls:
- `decay`: resonance length.
- `coef`: damping/brightness coefficient.

## Formant

Formant-filtered synthesis source for vowel-like spectra.

Source-specific controls:
- `formant`: formant center frequency.
- `bw`: formant bandwidth.

## Bowed

Bowed-string physical model.

Source-specific controls:
- `pressure`: bow pressure.
- `bow_pos`: bow position on the string.

## Blown

Blown pipe/wind physical model.

Source-specific controls:
- `pressure`: blowing pressure.
- `embouchure`: embouchure/tube excitation shape.

## Membrane

Struck membrane model.

Source-specific controls:
- `tension`: membrane tension.
- `loss`: energy loss per cycle.

## Marimba

Malleted bar resonator model.

Source-specific controls:
- `hardness`: mallet hardness.
- `bar_pos`: strike position along the bar.

## Vibes

Vibraphone-style resonator with motor modulation.

Source-specific controls:
- `motor_speed`: motor/tremolo speed.
- `damper`: damping amount.

## Kalimba

Thumb-piano style resonator.

Source-specific controls:
- `damping`: decay damping.
- `brightness`: high-frequency emphasis.

## Steel Drum

Steel pan resonator model.

Source-specific controls:
- `tone`: spectral brightness/voicing.
- `decay`: note decay time.

## Tubular Bell

Tubular bell/chime model.

Source-specific controls:
- `strike_pos`: strike position.
- `damping`: bell damping.

## Glockenspiel

Metal bar percussion model.

Source-specific controls:
- `hardness`: mallet hardness.
- `decay`: ring time.

## Guitar

Plucked-string model with body resonance.

Source-specific controls:
- `pick_pos`: pluck/pick position.
- `damping`: string damping.
- `body`: body resonance contribution.

## Bass Guitar

Electric bass model.

Source-specific controls:
- `finger`: finger/pluck character.
- `fret_noise`: fret/transient noise level.

## Harp

Harp-style plucked-string model.

Source-specific controls:
- `pedal`: pedal/tuning coloration control.
- `damping`: string damping.

## Koto

Koto-inspired plucked-string model.

Source-specific controls:
- `bend`: pitch bend amount.
- `damping`: string damping.
