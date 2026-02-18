# Drums

Dedicated drum and percussion source types.

Most drum entries use short envelopes and expose `amp` plus a small set of
instrument-specific controls.

## Kick

Synthesized bass drum source.

Source-specific controls:
- `freq`: base drum pitch.
- `punch`: pitch-envelope intensity.
- `click`: transient click level.
- `decay`: body decay time.

## Snare

Snare source combining tonal body and noise components.

Source-specific controls:
- `tone_freq`: tonal body frequency.
- `noise_amt`: noise component level.
- `snap`: transient snap intensity.
- `decay`: decay time.

## HiHat

Shared source for both Hi-Hat Closed and Hi-Hat Open variants (different
default decay values).

Source-specific controls:
- `tone`: hat brightness/center frequency.
- `decay`: ring time.

## Clap

Handclap-style transient source.

Source-specific controls:
- `spread`: timing spread between clap transients.
- `decay`: tail length.

## Cowbell

Metallic tuned percussion source.

Source-specific controls:
- `detune`: offset between partials.
- `decay`: ring time.

## Rim

Rimshot/cross-stick style source.

Source-specific controls:
- `tone`: body frequency.
- `click`: transient intensity.
- `decay`: decay time.

## Tom

Tuned tom drum source.

Source-specific controls:
- `pitch`: tom pitch.
- `decay`: ring time.

## Clave

Clave/wood-block style transient source.

Source-specific controls:
- `tone`: clave pitch/brightness.
- `decay`: decay time.

## Conga

Conga hand drum source.

Source-specific controls:
- `pitch`: conga tuning.
- `slap`: slap transient amount.
- `decay`: ring time.
