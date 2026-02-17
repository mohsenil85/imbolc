# The Signal Chain

Understanding how sound flows through an Imbolc track helps you shape the sound you want.

## Overview

Every track in Imbolc follows this signal path:

```
Source → LFO → Filter → EQ → Effects → Output
```

Each stage is optional (except the source). Sound flows left to right — the output of each stage feeds the input of the next.

## Source

The source generates the raw sound. This could be:
- A **synthesizer** (Saw, FM, SuperSaw, etc.) that creates sound from oscillator algorithms
- A **physical model** (Pluck, Bowed, Membrane) that simulates acoustic instruments
- A **sampler** (Pitched Sampler, Kit, Time Stretch) that plays back audio files
- An **input** (Audio In, Bus In) that brings in external or internal audio

The source's character is the foundation. Everything after it shapes, filters, and colors that foundation.

See the [Instruments Reference](../reference/instruments.md) for all 55 built-in source types.

## LFO (Low Frequency Oscillator)

An optional modulation source that creates movement by slowly varying a parameter over time. The LFO can target:
- Filter cutoff (creating wah-like sweeps)
- Amplitude (creating tremolo)
- Pitch (creating vibrato)
- Pan (creating auto-panning)

Toggle the LFO with `l` in the Track Editor. Cycle its shape with `s` (sine, triangle, saw, square, sample-and-hold, random) and its target with `m`.

## Filter

An optional frequency shaper. Filters remove or emphasize parts of the frequency spectrum.

Available filter types:
- **Low-Pass** — lets low frequencies through, cuts highs (warm, muffled)
- **High-Pass** — lets high frequencies through, cuts lows (thin, bright)
- **Band-Pass** — lets a narrow band through, cuts everything else
- **Notch** — cuts a narrow band, lets everything else through
- **Comb** — creates harmonic resonances
- **Allpass** — shifts phase without changing volume (used for spatial effects)
- **Vowel** — creates vowel-like formant sounds
- **ResDrive** — resonant filter with overdriven character

Toggle with `f` in the Track Editor. Cycle type with `t`. The two main controls are **cutoff** (which frequency to filter at) and **resonance** (how much the filter emphasizes frequencies near the cutoff).

## EQ (Equalizer)

A 12-band parametric equalizer for precise tonal shaping. Unlike the filter (which is a broad shaping tool), the EQ lets you surgically boost or cut specific frequency ranges.

Open the EQ pane with `F8`. Each band has frequency, gain, and Q (width) controls. The first band is a low shelf, the last is a high shelf, and the middle ten are peaking bands.

Toggle EQ on/off with `e` in the Track Editor.

## Effects Chain

Zero or more effects process the signal in series. The order matters — each effect receives the output of the one before it.

For example: Distortion → Reverb sounds different from Reverb → Distortion. The first adds distortion to a clean signal then reverberates it. The second reverberates first, then distorts the entire reverb tail.

Reorder effects with `Ctrl+Up`/`Ctrl+Down` in the Track Editor.

See the [Effects Reference](../reference/effects.md) for all 39 built-in effects.

## Output

The final signal is sent to an **output target**:
- **Master** — goes directly to the main output (your speakers/headphones)
- **Bus 1-8** — goes to a mixer bus for group processing

Buses allow you to apply shared effects to multiple tracks. For example, you might send several drum tracks to Bus 1 with a compressor on it. See [The Mixer](the-mixer.md) for more on routing.

## Why Order Matters

Think of the signal chain as a series of transformations. Each stage builds on what came before:

1. The **source** creates raw material
2. The **LFO** adds movement
3. The **filter** shapes the frequency content
4. The **EQ** fine-tunes the tonal balance
5. **Effects** add character, space, and texture
6. The **output** determines where it goes

Understanding this flow helps you diagnose problems ("why does my reverb sound distorted?" — maybe distortion is after the reverb) and make better creative decisions.

The processing chain order is also customizable: use `Ctrl+Up`/`Ctrl+Down` in the Track Editor to move stages (filter, EQ, effects) relative to each other.
