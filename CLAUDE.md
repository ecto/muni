# CLAUDE.md

This document provides an overview of the Muni repository for AI assistants.

## Project Overview

Muni (Municipal Robotics) is building machines that make the next machine: a six-axis arm (the limb), two limbs at any table (a pair), and the Rose humanoid, on a stack of our own. The plan, its stages (Desk → Shop → Road → City → Home → Moon → Mars → Star) and its products are in the master plan, `paper/master-plan/master-plan.typ`.

The BVR rover era is archived (see `ARCHIVE.md`): rover firmware and MCU code live at tag `archive/bvr-2026-10`, and the depot moved to `ecto/depo`.

## Repository Structure

```
muni/
├── paper/              # Technical documents (Typst)
│   ├── master-plan/    # The current plan: master-plan.typ, refs.bib, decisions.md, images
│   └── ...             # Earlier rover-era documents (manuals, whitepaper, deck)
├── web/                # muni.works (Next.js 14, static export, deployed by Vercel)
│   ├── posts.json      # Blog posts: slug, title, Typst source
│   ├── scripts/build-posts.mjs   # Typst → HTML post + PDF (npm run posts)
│   ├── src/content/posts/        # Generated post HTML (committed)
│   └── public/research/              # Generated post PDFs (committed)
├── hardware/           # Hardware concepts and mockups (loon sources, renders)
├── design/             # Design system
├── docs/               # Notes and architecture
└── video/              # Video tooling
```

## Master plan (paper/master-plan)

```bash
cd paper/master-plan
make                     # builds master-plan.pdf (Typst 0.14, fonts in fonts/)
```

- One source, two outputs: `--input target=html` builds the web post. Guard print-only show rules with `if web`.
- Product names are colored by family (`fam`, `elements`, `brand-names` near the top). The PDF colors them with a show rule; on the web, `build-posts.mjs` wraps them in `.name.n-<family>` spans.
- Equations inside drawn figures must carry `alt: "in-frame"` (figures and `mathfig` set it) so HTML export keeps them.
- No page may start with a figure or table. Check with `typst query --font-path fonts --input figpos=1 master-plan.typ "<figpos>" --field value` (any figure with y < 110 pt is at a page top).
- Record decisions in `decisions.md` (D1, D2, …).

## Research posts (web)

```bash
cd web
npm install
npm run dev              # http://localhost:3000/research
npm run posts            # rebuild every post from its Typst source (needs typst on PATH)
npm run build            # static export
```

Vercel has no typst, so generated HTML and PDFs are committed. Never run `next build` while the dev server is running (it corrupts `.next`).

## Conventions

### TypeScript (web)
- Use `const` enums with `as const` for type-safe constants
- Prefer interfaces over types for object shapes
- Components in `src/components/`, pages in `src/app/`

### Git
- LFS is configured for large binary files (CAD, images, PDFs). Run `git lfs pull` after cloning.
- Web assets in `web/` are excluded from LFS for GitHub Pages compatibility. Verify with `git check-attr filter <file>`.
