# Robots that build robots — vision whitepaper

25-page whitepaper in the dark muon-neon design system (IBM Plex Sans/Mono on a near-black page).
Source is `vision.typ`; `vision.pdf` is the committed build.

## Build

```bash
cd paper/vision
make            # -> vision.pdf
make png        # page PNGs in out/ (gitignored scratch)
make watch      # live rebuild
```

Needs [typst](https://typst.app) 0.13+ (`brew install typst`). The `@preview/cetz` and
`@preview/fletcher` packages are fetched by typst on first build. Fonts come only from `fonts/`
(IBM Plex, SIL OFL 1.1, see `fonts/OFL-IBM-Plex.txt`); system fonts are ignored so builds are
reproducible.

## Images

- `images-native/` — renders made directly in the dark theme (`vcad-render --theme dark`); the
  generators live in `hardware/` at the repo root.
- `images-dark/` — light renders re-toned onto the page palette by `darken.py`
  (`make darken`, needs numpy + Pillow). Only the source used by the paper is kept in `images/`.

Only the images the paper references are committed.
