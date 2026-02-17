# Your First Session

_Time: ~15 minutes_

By the end of this tutorial you will have a short looping melody playing through
a synthesizer, with a filter and an effect shaping the sound. No previous
experience with Imbolc is needed -- just a working installation
([Installation](../how-to/installation.md)).

---

## What you will build

A four-bar phrase using a synthesizer, looping in real time. You will shape
the timbre with a low-pass filter and add some reverb to give the sound space.
Along the way you will learn the core workflow: create a track, enter notes,
set up a loop, and tweak the sound.

---

## 1. Launch Imbolc

From the repository root:

```bash
cargo run -p imbolc-ui --release
```

Imbolc requires a terminal that supports the Kitty keyboard protocol.
**Kitty** and **Ghostty** both work. If keys behave strangely, switch to one
of those terminals before continuing.

You will land on the **Home** screen.

## 2. Start the audio server

Press **F5** to open the **Server** pane. This is where you control
SuperCollider's audio engine.

1. Press `s` to **start** the server.
2. Press `c` to **connect** Imbolc to the running server.

If this is your first time running Imbolc (or after updating):

3. Press `b` to **build** (compile) the SynthDefs.
4. Press `l` to **load** the compiled SynthDefs into the server.

The status line in the Server pane will confirm each step. Once connected and
loaded, you are ready to make sound.

## 3. Create a track

Press **F1** to open the **Instruments** pane. This lists all your tracks.
It will be empty right now.

1. Press `a` to **add** a new track.
2. A source browser appears. Use the **arrow keys** to scroll through the
   available sound sources. Good starting points:
   - **Saw** -- a bright, classic sawtooth synthesizer
   - **EPiano** -- a warm electric piano
   - **Pluck** -- a plucked string model
3. Press **Enter** to confirm your choice.

Your new track appears in slot 1. Its name and source type are shown in the
list. You can select tracks at any time by pressing their slot number
(`1`--`9`, `0` for slot 10).

## 4. Write a melody

Press **F2** to open the **Piano Roll**. This is a grid where pitch runs
vertically (higher notes toward the top) and time runs horizontally (left to
right).

1. Use the **arrow keys** to move the cursor to where you want your first note.
   - **Up / Down** -- change pitch
   - **Left / Right** -- move through time
   - **PageUp / PageDown** -- jump up or down by an octave
2. Press **Enter** to **place a note** at the cursor. Press Enter again on the
   same cell to remove it.
3. Place a few more notes to create a short phrase -- four to eight notes is
   plenty.
4. To adjust a note's velocity (how hard the note is played):
   - Move the cursor onto the note.
   - Press `+` to increase velocity, `-` to decrease it.
5. To change how long each note lasts:
   - Press **Alt+Right** to grow the note duration.
   - Press **Alt+Left** to shrink it.

Tip: press `z` to zoom in on the time axis, or `x` to zoom out, so you can
see more or fewer beats at once.

## 5. Set up a loop

Looping lets you hear your phrase repeat while you keep editing.

1. Press **Home** to jump the cursor to the very start of the timeline.
2. Press `[` to **set the loop start** at the cursor position.
3. Use the **Right** arrow to move the cursor to just past the end of your
   phrase.
4. Press `]` to **set the loop end**.
5. Press `l` to **enable looping**.

A highlighted region in the piano roll marks the loop boundaries.

## 6. Play it back

Press **Space** to start playback. You should hear your melody looping. Press
**Space** again to stop.

If you hear nothing, jump back to the Server pane (**F5**) and check that the
server is started and connected. See [Common first-run issues](#common-first-run-issues)
at the bottom of this page.

While playback is running, you can continue editing notes in the piano roll --
changes take effect immediately.

## 7. Shape the sound

Now make the sound more interesting. Press **F1** to return to the Instruments
pane, then press **Enter** to open the **Track Editor** for the selected track.

The Track Editor is organized into sections. Press **Tab** to cycle between
them (source parameters, filter, envelope, effects, and more). Use
**Shift+Tab** to go back a section.

### Turn on a filter

1. Press `f` to **toggle the filter on**.
2. Press `t` to **cycle the filter type** (Low-pass, High-pass, Band-pass,
   Notch). Start with Low-pass.
3. Navigate to the filter cutoff parameter with the **arrow keys**.
   - **Left / Right** adjusts the value.
   - **PageUp / PageDown** adjusts in larger steps.

Sweep the cutoff down to darken the sound, or up to brighten it. Play with
the resonance parameter the same way.

### Add an effect

1. Press `a` to **add an effect**. A browser opens.
2. Use the arrow keys to browse. Try **Reverb** for space, or **Delay** for
   rhythmic echoes. Press **Enter** to confirm.
3. The effect's parameters appear in the effects section. Navigate with the
   arrow keys and adjust values with **Left / Right**.

You can add more effects by pressing `a` again, or remove one with `d`.
Toggle an effect's bypass with `b` to compare the sound with and without it.

### Adjust the envelope

Tab to the **Envelope** section. The ADSR controls (Attack, Decay, Sustain,
Release) shape how each note fades in and out. A longer attack creates a
swell; a longer release lets notes ring out. Adjust each value with the arrow
keys.

## 8. Save your work

Press **Ctrl+s** to save your project. The default save location is:

```
~/.config/imbolc/default.sqlite
```

To save under a different name, press **Ctrl+Shift+s** (Save As) and type a
filename.

To reopen a saved project later, press **Ctrl+o** for the project browser, or
**Ctrl+l** to load directly.

## 9. Next steps

You have created a track, written a melody, shaped the sound, and saved the
project. Here are some directions to explore next:

- **[Making a Beat](making-a-beat.md)** -- add drums and a bass line, mix
  them together, and export audio.
- **Effects** -- Imbolc has 39 built-in effects. Experiment with Chorus,
  Distortion, or Phaser.
- **Piano mode** -- press `/` to enter piano keyboard mode, where letter
  keys become a playable keyboard. Press `Escape` to cycle layout or exit.
- **Context help** -- press `?` anywhere to see the keybindings for the
  current pane.
- **Command palette** -- press `:` to search and run commands by name.

---

## Common first-run issues

**No sound**
Open the Server pane (**F5**). Check that the status shows "Running" and
"Connected". If not, press `s` to start, then `c` to connect.

**Missing SynthDefs / "no SynthDef" errors**
In the Server pane, press `b` to build (compile) SynthDefs, then `l` to
load them. This only needs to be done once after installation or updates.

**Keys behaving strangely**
Imbolc relies on the Kitty keyboard protocol for accurate key detection.
Use **Kitty** or **Ghostty** as your terminal emulator. Other terminals
(Terminal.app, iTerm2, GNOME Terminal) may not report key events correctly.

**Jerky playback or audio glitches**
In the Server pane, press **Tab** to cycle to the device/buffer settings. Try
increasing the buffer size for more stable playback at the cost of slightly
higher latency.
