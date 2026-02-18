# Routing

Sources that route external or internal audio into a track.

## Audio In

Captures hardware audio input and feeds it through the standard track signal
chain.

Controls:
- `channel`: hardware input channel index.
- `gain`: input gain multiplier.
- `test_tone`: enables an internal sine test tone.
- `test_freq`: test tone frequency.
- `fb_suppress`: feedback suppression amount/enable.
- `fb_threshold`: suppression detection threshold.
- `fb_q`: suppression notch width (Q-related control).

## Bus In

Reads audio from an internal bus and processes it as a normal track source.

Controls:
- `bus`: logical bus selector.
- `gain`: input gain multiplier.
