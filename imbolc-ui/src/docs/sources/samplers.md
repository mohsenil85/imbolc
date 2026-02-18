# Samplers

Sample playback and grain-based source types.

## Pitched Sampler

Plays an audio buffer as a pitched instrument voice.

Core controls:
- `rate`: playback speed and transposition.
- `amp`: source output level.
- `loop`: loop enable/disable.

Related data controls:
- sample/buffer selection.
- `sliceStart`, `sliceEnd`: normalized playback range within the buffer.

## Time Stretch

Granular sample player that separates timing (`stretch`) from pitch (`pitch`).

Core controls:
- `stretch`: playback time scale.
- `pitch`: transposition in semitones.
- `grain_size`: grain duration.
- `overlap`: grain overlap count.
- `amp`: source output level.

Related data controls:
- sample/buffer selection.
- `sliceStart`, `sliceEnd`: normalized playback range within the buffer.

## Kit

Twelve-pad one-shot sampler used by the drum sequencer.

Per-pad controls:
- `level`: pad output level.
- `pitch`: semitone offset.
- `reverse`: reverse playback toggle.
- `slice_start`, `slice_end`: sample region per pad.
- optional instrument trigger assignment (pad can trigger another track).

## Granular

Real-time granular oscillator using sine grains (not sample-buffer granulation).

Core controls:
- `freq`: center pitch.
- `grain_size`: grain duration.
- `density`: grain trigger rate.
- `spread`: stereo/random pan spread.
- `pitch_rnd`: per-grain random pitch deviation.
- `amp`: source output level.
