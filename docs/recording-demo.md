# Recording a TUI Demo for YouTube

## Overview

Pipeline: **Prep terminal → Record with OBS → Edit in DaVinci Resolve → Export & Upload**

---

## 1. Terminal Prep

- **Font size**: 20-24pt so text is readable on YouTube
- **Clean up**: Clear scrollback, hide unnecessary tabs/chrome
- **Window size**: Maximize or set a specific size that fills the recording canvas
- **Notifications**: Enable Focus Mode on macOS to block popups during recording
- **Theme**: Use a dark theme with good contrast

---

## 2. Audio Capture Setup

Since Imbolc uses SuperCollider for audio, you need to route SC's output to both your speakers and your recording software.

### Native Method (macOS 13+, OBS 30+)

No extra software needed. In OBS:

1. Sources → **+** → **macOS Audio Capture**
2. Select "Capture all desktop audio" or pick SuperCollider specifically

### BlackHole Fallback (older macOS)

Install [BlackHole](https://github.com/ExistentialAudio/BlackHole), a free zero-latency audio loopback driver:

```bash
brew install blackhole-2ch
```

Create a Multi-Output Device:

1. Open **Audio MIDI Setup** (`/Applications/Utilities/`)
2. Click **+** at bottom-left → **Create Multi-Output Device**
3. Check both:
   - Your speakers/headphones (**must be first** — this is the clock source)
   - **BlackHole 2ch**
4. Set as system output: **System Settings → Sound → Output → Multi-Output Device**

In OBS: Sources → **+** → **Audio Input Capture** → select **BlackHole 2ch**

> **Note:** The Multi-Output Device has no volume control in the menu bar. Adjust volume before setting it as default, or control volume from within apps.

---

## 3. OBS Setup

### Installation

```bash
brew install --cask obs
```

### Video Source

1. Sources → **+** → **macOS Screen Capture**
2. Change mode to **Window Capture**
3. Select your terminal window
4. Right-click the source → **Transform** → **Fit to Screen**

### Output / Recording Settings

**Settings → Output**, set Output Mode to **Advanced**, then the **Recording** tab:

| Setting | Value | Notes |
|---------|-------|-------|
| Type | Standard | |
| Recording Format | mkv | Won't corrupt if OBS crashes; remux to mp4 after |
| Encoder | Apple VT H265 (or x264) | Hardware encoder on Apple Silicon is fast |
| Rate Control | CRF | Constant quality, ideal for local recording |
| CRF | 18 | Near-lossless with reasonable file size |

After recording, use **File → Remux Recordings** to convert `.mkv` → `.mp4`.

### Video Settings

**Settings → Video:**

| Setting | Value |
|---------|-------|
| Base Resolution | Match your display (e.g. 2560x1440) |
| Output Resolution | 1920x1080 or 2560x1440 |
| Downscale Filter | Lanczos |
| FPS | 60 |

> **Tip:** 1440p is noticeably sharper for text. YouTube serves 1440p with a higher bitrate than 1080p.

### Audio Settings

**Settings → Audio:**

| Setting | Value |
|---------|-------|
| Sample Rate | 48 kHz |
| Channels | Stereo |

In the **Audio Mixer** dock, set levels so audio peaks around **-6 to -3 dB**.

### Pre-Recording Checklist

1. Open OBS, verify sources look right in the preview
2. Start SuperCollider / Imbolc
3. Play a note — check audio meters are moving
4. Do a **10-second test recording**, play it back to verify video + audio
5. Record the real demo
6. Remux `.mkv` → `.mp4` via **File → Remux Recordings**

---

## 4. Video Editing (DaVinci Resolve)

### Installation

```bash
brew install --cask davinci-resolve
```

Free version covers everything needed. Alternative: iMovie (already on your Mac) if you just need basic trimming.

### Editing Workflow

**Import & Rough Cut:**
- Import the remuxed `.mp4` into the **Media Pool**
- Drag onto the timeline on the **Edit** page
- Trim start and end
- Cut dead time or mistakes with **Blade tool** (`B`) → select and delete

**Demo Structure:**

1. **Hook** (0-15s) — show the most impressive thing first (e.g., a beat playing)
2. **Brief intro** (15-30s) — what is Imbolc, one sentence
3. **Walkthrough** — features one by one, each as a short segment
4. **Wrap** — where to find it (GitHub link), call to action

**Text / Titles:**
- Edit page → **Effects Library** → **Titles** → drag "Text" onto a track above your video
- Use for section headers, keybinding callouts, or a title card
- Keep fonts clean and readable (system sans-serif, white on dark)

**Audio Polish:**
- Switch to the **Fairlight** page
- Apply a **Compressor** to even out volume (Effects → Audio FX → Dynamics → Compressor)
- Normalize loudness to **-14 LUFS** (YouTube's target)
- If adding voiceover, record separately and layer on a second audio track

**Contrast Boost (optional):**
- On the **Color** page, bump contrast slightly so terminal text pops
- Terminal recordings can look washed out after YouTube re-encodes

---

## 5. Export Settings for YouTube

On the **Deliver** page, use the **YouTube** preset and tweak:

| Setting | Value |
|---------|-------|
| Format | MP4 |
| Codec | H.265 (or H.264 for wider compatibility) |
| Resolution | 1920x1080 or 2560x1440 |
| Frame Rate | 60 |
| Quality | 20,000-40,000 kbps (or Automatic → Best) |
| Audio | AAC, 320 kbps |

DaVinci can upload directly to YouTube from the Deliver page if you link your account.

---

## 6. YouTube Upload Tips

- **Upload at 1440p if possible** — higher bitrate allocation means sharper text
- **Wait for processing** — the high-quality version (VP9/AV1) can take 30-60 minutes to appear
- **Add chapters** — in the description, add timestamps:
  ```
  0:00 Intro
  0:30 Sequencer
  1:15 Mixer
  2:00 Effects
  ```
- **Thumbnail** — screenshot the most visually interesting state of Imbolc, add a title overlay (Canva works well for this)

---

## Alternative Recording Tools

| Tool | Best For | Notes |
|------|----------|-------|
| [Screen Studio](https://screen.studio/) | Polished product demos | Paid macOS app, auto-zoom, minimal effort |
| [VHS](https://github.com/charmbracelet/vhs) | Scripted/reproducible clips | `brew install vhs`; uses `.tape` files; may struggle with complex TUI input |
| [Asciinema](https://asciinema.org/) + [Remotion](https://remotion.dev/) | Sharpest text rendering | Most setup, but browser-rendered text is very crisp |
| macOS built-in | Quick & dirty | Cmd+Shift+5; no system audio capture |
| iMovie | Simple edits | Already installed, minimal learning curve |
