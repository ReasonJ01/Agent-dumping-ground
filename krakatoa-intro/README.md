# Krakatoa intro

A 32 second, MGSV-style mission opening for the Krakatoa squad's end of sprint demos
(sprint 3990, "Futalognkosaurus"). Built with [fframes](https://github.com/dmtrkovalenko/fframes)
(Rust + SVG, rendered with Skia).

The rendered video is [`krakatoa-intro.mp4`](krakatoa-intro.mp4) (1080p30, re-encoded to 20 MB).

| time | scene | what happens |
| --- | --- | --- |
| 0.0–4.5s | `ColdOpen` | "Incoming transmission": squad, sprint dates and "End of sprint demos" typed out |
| 4.5–13.5s | `MissionArea` | Topographic map; each sprint goal drops in as a target marker, LBG Gem API as a side op |
| 13.5–24.5s | `Credits` | The squad credited MGSV style, names glitching in and out |
| 24.5–32s | `EpisodeTitle` | "EPISODE 3990 / FUTALOGNKOSAURUS" slams in, holds, tears out |

## Building

This project uses the fframes workspace's dependencies, so it builds from inside a checkout of
the fframes repository (tested against commit `654ae86`):

```bash
git clone https://github.com/dmtrkovalenko/fframes
cp -r krakatoa-intro fframes/examples/
# add "examples/krakatoa-intro" to `members` in fframes/Cargo.toml
cd fframes
# Debian/Ubuntu: FFmpeg is compiled from source and needs the codec libraries
sudo apt-get install -y nasm pkg-config libx264-dev libx265-dev libvpx-dev
cargo build --release -p krakatoa-intro
```

## Commands

Run from `fframes/examples/krakatoa-intro` (`R=../../target/release/krakatoa-intro`):

```bash
$R timeline                     # scenes and audio cues
$R inspect --pops               # automatic problem checks
$R strip Credits -n 12          # contact sheet of a scene
$R render -o out/krakatoa-intro.mp4
$R --renderer gpu render        # much faster on a machine with a GPU (Vulkan)
```

It defaults to Skia on the CPU because cloud containers have no GPU. That works but is slow:
the map and film grain are shaders evaluated on every frame.

## Assets

- Fonts: Anton, IBM Plex Mono, Inter (SIL Open Font License), copied from the fframes examples.
- Grain and contour shaders, and the helpers for glitches, crop marks and callouts, are adapted
  from the fframes `fframes-intro` example (MIT).
- `dynamic_media/sfx_*.mp3` are copied from the fframes `fframes-intro` example; their licence
  is not stated there, so check before publishing the video outside the team.
- `dynamic_media/drone.wav` is a placeholder hum generated with ffmpeg. Swap in real music.
