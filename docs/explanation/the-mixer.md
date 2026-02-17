# The Mixer

The mixer is where individual track signals combine into a final output. Open it with `F4`.

## Mixer Channels

Each track in your project has a corresponding mixer channel. A channel provides:
- **Level** — volume control (Up/Down to adjust)
- **Pan** — stereo position (`p`/`P` to pan left/right)
- **Mute** — silence a channel (`m`)
- **Solo** — hear only this channel (`s`)

## Output Targets

Every track routes to an output target:
- **Master** — the main stereo output (what you hear through your speakers)
- **Bus 1-8** — internal mixing buses for group processing

Press `o` on a mixer channel to cycle its output target. Tracks routed to a bus are summed together and processed by that bus's effect chain before reaching the master.

## Buses

Buses are submix groups. They're useful for:
- **Shared effects** — put a reverb on Bus 1, route several tracks to it, and they all share the same reverb rather than each having their own
- **Group processing** — compress all your drums together by routing them to a bus with a compressor
- **Submixing** — control the overall level of a group of tracks with one fader

Each bus has its own level, pan, mute/solo, and effect chain. Navigate to bus channels in the mixer by scrolling past the track channels.

## Sends

Sends let a track feed signal to a bus *in addition to* its main output. This is different from routing: a track routed to Bus 1 goes *only* to Bus 1, while a track with a send to Bus 1 goes to both its main output *and* Bus 1.

The classic use is reverb: keep tracks routed to Master, but add sends to a bus with reverb. Each track's send level controls how much reverb it gets.

### Tap Points

Each send has a tap point that controls *where* in the signal chain the send picks up:
- **Post-insert** (default) — the send receives the signal after the track's effects chain
- **Pre-insert** — the send receives the signal before effects processing

## Layer Groups

Layer groups let you link multiple tracks so they play together. When you play a note, all tracks in the group sound simultaneously. Each track in the group can have a different octave offset.

Link tracks with `l` in the Instruments pane. Unlink with `L`.

Layer groups have their own mixer channel with level, pan, and effects.

## Master

The master channel is the final output stage. All tracks and buses ultimately feed into the master. It has:
- **Level** — overall output volume
- **Mute** — silence everything (or press `.` globally)

The master channel appears at the right end of the mixer.

## Signal Flow Summary

```
Track 1 ──────────────────────┐
Track 2 ──→ Bus 1 (effects) ──┤
Track 3 ──→ Bus 1             ├──→ Master ──→ Speakers
Track 4 ──────────────────────┤
Track 5 ──→ Bus 2 (effects) ──┘
```

Tracks can route directly to master or through buses. Sends add additional routing paths without changing the main route.
