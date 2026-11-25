# Koma media

Generated images for [Koma](https://github.com/Anthonyy232/Koma), kept off `main` so it stays light: the app icon, the README hero and feature panels, the social preview and the release banners. Every script takes the path of a Koma checkout of `main`, reads the logo from its `assets/`, and needs no network.

| Path | What |
| --- | --- |
| `lib.cjs` | Fonts and rendering shared by the scripts |
| `fonts/` | Instrument Serif, under the SIL Open Font License (`OFL.txt`) |
| `readme/demo.html` | The invented desktop Koma is captured over (`?dark` for the dark one) |
| `readme/capture.cjs` | Shows the demo desktop full screen and drives Koma over it with `tools/peek`, into `readme/inputs/` |
| `readme/build.cjs` | Writes `windows/koma.ico` and `.github/readme/*.png` in the checkout, and `social-preview.png` here |
| `social-preview.png` | The 1280x640 GitHub social preview |
| `releases/banner.cjs` | Writes `releases/v<version>.png` from `releases/banners.json` |

## Setup

From a Koma checkout of `main`:

```bash
git worktree add local/media media
cd local/media
npm ci && npx playwright install chromium
```

## Icon, README images and social preview

```bash
node readme/build.cjs ../..
```

The icon holds 16 to 256 px images: `assets/logo-small.svg` up to 32 px and `assets/logo.svg` above.

Commit `social-preview.png` here and the changed images on `main`. GitHub has no API for the social preview: upload `social-preview.png` in the repository settings under Social preview.

## Screenshots

Only when the interface changed. Koma pictures the real desktop, so the capture takes over the primary display for a few seconds for each scheme: it opens `readme/demo.html` full screen in Chromium, starts Koma, selects an area, draws an arrow, an ellipse, a pixelated block and a text note, and saves the display's middle 1600x1000 points as `select-<scheme>.png` and `marks-<scheme>.png`. Keep your hands off the mouse and keyboard while it runs.

```bash
(cd ../.. && cargo build -p koma-windows -p peek)   # with the Windows SDK's fxc on PATH
node readme/capture.cjs ../.. [<left> <top>]
```

`<left> <top>` is the primary display's top left within the whole desktop, in pixels: `0 0`, the default, unless a display sits to its left or above it (`2560 0` with a 2560 px wide display on the left). The primary display must be at least 1600x1000 points. The committed inputs were taken at 125% scale, so they are 2000x1250; pixelation's noise differs on every capture, so a new capture never matches the last one pixel for pixel.

## Release banners

```bash
node releases/banner.cjs ../.. 0.1.0 "One short highlight"
```

That records the highlight in `releases/banners.json` and writes `releases/v0.1.0.png` (1280x640 at 2x). Without a version it redraws every banner. Commit, push, and use the raw URL in the release notes:

`https://raw.githubusercontent.com/Anthonyy232/Koma/media/releases/v0.1.0.png`
