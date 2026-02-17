# Making a Beat

_Time: ~20 minutes_

By the end of this tutorial you will have a drum pattern, a bass line, a basic
mix, and an exported WAV file. This picks up where
[Your First Session](first-session.md) left off.

---

## What you will build

A simple beat with a kick-and-snare drum pattern, a repeating bass line, and a
rough mix balanced between the two. You will add some effects to give the drums
punch and the bass character, then bounce the result to a WAV file you can
share or import into another tool.

---

## Prerequisites

- You have completed [Your First Session](first-session.md).
- The audio server is running and connected (Server pane, **F5** -- status
  shows "Running" and "Connected").

If you still have the track from the first tutorial, that is fine -- you will
add new tracks alongside it.

---

## 1. Add a drum track

Press **F1** to open the **Instruments** pane.

1. Press `a` to add a new track.
2. In the source browser, scroll to **Kit** and press **Enter**.

A Kit track is different from a melodic track. Instead of notes on a piano
roll, it has a set of pads -- each pad holds a different drum sound. Kits come
pre-loaded with a basic set (kick, snare, hi-hat, and more) that you can
replace with your own samples later.

## 2. Program a drum pattern

With the Kit track selected (check the slot number -- press the corresponding
number key if needed), press **F2** to open the **Piano Roll**. Because the
selected track is a Kit, press `t` to switch to the **step sequencer view**.

The step sequencer shows a grid of pads (rows) and steps (columns). Each row
is a different drum sound; each column is a point in time.

### Layout

- **Up / Down** arrow keys move between pads.
- **Left / Right** arrow keys move between steps.
- **Enter** toggles a step on or off.

### Build the pattern

Start with a classic four-on-the-floor pattern:

1. Select the **Kick** pad row.
2. Toggle on steps 1, 5, 9, and 13 (beats 1, 2, 3, and 4 in a 16-step bar).
3. Move to the **Snare** pad row.
4. Toggle on steps 5 and 13 (beats 2 and 4).
5. Move to the **HiHat** pad row.
6. Toggle on every other step (1, 3, 5, 7, 9, 11, 13, 15) for eighth notes.

Press `g` to **cycle the step resolution** if you want finer divisions (1/4,
1/8, 1/16, 1/32 notes).

### Preview

Press **Space** to start playback and hear the pattern. Adjust steps while it
plays -- changes are immediate.

### Load custom samples

To replace a pad's sound with your own sample:

1. Move the cursor to the pad you want to change.
2. Press `s` to open the **file browser**.
3. Navigate to a WAV or AIFF file and press **Enter** to load it.

You can also use the **sample chopper** -- press `c` to slice a longer sample
into individual hits and assign them to pads.

### Adjust individual pads

- **Ctrl+Left / Ctrl+Right** adjusts a pad's level.
- `r` toggles **reverse** playback for the selected pad.
- `=` / `-` shifts the pad's pitch up or down by a semitone.
- `+` / `_` adjusts the velocity of the step under the cursor.

## 3. Add a bass line

You need a second track for bass.

1. Press the number key for an empty slot (e.g. `2` or `3`) or press `>` to
   move to the next available track.
2. Press **F1**, then `a` to add a new track.
3. Choose a bass-friendly source:
   - **Saw** -- classic subtractive bass
   - **Acid** -- squelchy resonant character
   - **BassGuitar** -- physical-modeled upright bass
4. Press **Enter** to confirm.

Now write the bass pattern:

1. Press **F2** to open the Piano Roll. Since this is a melodic track, you
   will be in the note editor view (not the step sequencer).
2. Press **PageDown** a few times to scroll down to a bass register (around
   C2 or C3).
3. Use the arrow keys to position the cursor and **Enter** to place notes.
   Keep it simple -- root notes on the beat, or a short repeating riff.
4. Set the loop:
   - **Home** to jump to the start, then `[` to set loop start.
   - Move to the end of your phrase, then `]` to set loop end.
   - Press `l` to enable looping.
5. Press **Space** to hear the bass alongside the drums.

## 4. Mix it together

Press **F4** to open the **Mixer**. You will see a channel strip for each
track, plus a master channel on the right.

### Navigate

- **Left / Right** arrow keys move between channels.
- **Home / End** jump to the first or last channel.
- **Tab** cycles between mixer sections (faders, sends, detail).

### Balance levels

- **Up / Down** arrow keys raise or lower the selected channel's level.
- **PageUp / PageDown** adjust in larger increments.

Pull the drum track up or down until it sits well against the bass. There is no
"correct" level -- trust your ears.

### Pan

- Press `p` to nudge the selected channel's pan **left**.
- Press `P` (Shift+p) to nudge it **right**.

Try panning the hi-hat slightly to one side for width, or keep everything
centered for a focused sound.

### Mute and solo

- `m` toggles **mute** on the selected channel (silence it).
- `s` toggles **solo** (hear only this channel).

Use solo to focus on one element while adjusting its level or effects, then
un-solo to hear the full mix again.

## 5. Add some effects

### Give the drums punch

1. Select the drum track (press its number key, e.g. `1`).
2. Press **F1**, then **Enter** to open the Track Editor.
3. Press `a` to add an effect.
4. Choose **TapeComp** (tape-style compressor) and press **Enter**.
5. Navigate to the compressor's parameters with the arrow keys. Adjust the
   threshold and ratio to taste -- even the defaults will add some glue.

### Shape the bass

1. Select the bass track (press its number key).
2. Press **F1**, then **Enter** to open the Track Editor.
3. Press `f` to toggle the **filter** on. Press `t` to set it to **Low-pass**.
   Sweep the cutoff down to tame the highs.
4. Press `a` to add an effect. Try:
   - **Distortion** for grit and presence.
   - **Saturator** for warmer harmonic color.
5. Adjust the effect parameters with the arrow keys.

Press **Space** to play back the full mix with effects. Toggle an effect's
bypass with `b` in the Track Editor to compare the sound with and without it.

## 6. Fine-tune

A few more tools that help at this stage:

- **Click track**: press `M` (Shift+m) anywhere to toggle the metronome. It
  helps you check that your pattern sits on the beat.
- **Groove**: press **F9** to open groove settings. Adjust swing to give the
  drums a looser, more human feel.
- **Undo**: press **Ctrl+z** at any time to step back. **Ctrl+Shift+z**
  to redo.

## 7. Export your beat

When you are happy with the mix, export it as audio.

1. Press **F2** to return to the Piano Roll.
2. Press `B` (Shift+b) to **bounce the master** output to a WAV file.

The file is saved to:

```
~/.config/imbolc/exports/
```

For individual stems (one WAV per track), press **Ctrl+b** instead.

To render just the selected track, press `R`.

Saved files are standard WAV format, ready to import into any other audio
tool.

## 8. Save your project

Press **Ctrl+s** to save. Your drum patterns, bass line, effects, and mix
settings are all preserved in the project file.

---

## Next steps

You now have the fundamentals: sequencing drums, writing melodic parts, mixing,
and exporting. Here are some directions to continue:

- **More tracks** -- add pads, leads, or atmospheric textures. Imbolc supports
  dozens of built-in sources, from physical models (Bowed, Blown, Membrane) to
  FM synthesis (FM, FMBell, FMBrass).
- **Automation** -- press **F7** to open the automation editor. Draw curves to
  change parameters over time (filter sweeps, volume fades, effect sends).
- **Arrangement** -- press **F3** to open the Track (arrangement) view.
  Capture your loops as clips and arrange them into a full song structure.
- **EQ** -- press **F8** for a 12-band parametric EQ. Carve space for each
  element in the frequency spectrum.
- **Piano mode** -- press `/` to turn your keyboard into a playable
  instrument. Great for auditioning sounds or recording parts in real time.
- **Context help** -- press `?` in any pane to see all available keybindings
  for that context.
