# Recording and Export

How to capture audio from Imbolc.

## Master Recording (Real-Time)

Records everything you hear to a WAV file as you work.

1. Press `Ctrl+r` to start recording (or `R` in the Server pane).
2. Play, jam, tweak — everything going through the master bus is captured.
3. Press `Ctrl+r` again to stop.
4. Output: `master_<timestamp>.wav` in your current working directory.

## Single Track Render

Renders one track's audio offline (faster than real-time).

1. Open the Piano Roll (`F2`).
2. Press `R`.
3. Output: `~/.config/imbolc/renders/render_<track>_<timestamp>.wav`.

## Master Bounce

Bounces the full mix offline.

1. Open the Piano Roll (`F2`).
2. Press `B`.
3. A progress indicator appears while the bounce runs.
4. Output: `~/.config/imbolc/exports/bounce_<timestamp>.wav`.

## Stem Export

Exports each track as a separate WAV file.

1. Open the Piano Roll (`F2`).
2. Press `Ctrl+b`.
3. Each track is rendered individually.
4. Output: `~/.config/imbolc/exports/stem_<name>_<timestamp>.wav`.
