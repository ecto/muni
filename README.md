<picture>
  <source media="(prefers-color-scheme: dark)" srcset="web/public/images/muni-logo-dark.svg">
  <source media="(prefers-color-scheme: light)" srcset="web/public/images/muni-logo-light.svg">
  <img alt="muni" src="web/public/images/muni-logo-light.svg" width="120">
</picture>

Open-source municipal robotics: machines that make the next machine, starting with arms, pairs and the Rose humanoid. The plan is in [the vision](https://muni.works/research/vision) ([source](paper/vision/vision.typ)).

## Repository

```
muni/
├── paper/      # Technical documents (Typst); vision/ is the current plan
├── web/        # muni.works (Next.js); the blog is compiled from paper/ by Typst
├── hardware/   # Hardware concepts and mockups (loon sources, renders)
├── design/     # Design system
├── docs/       # Notes and architecture
└── video/      # Video tooling
```

The BVR rover era is archived; see [ARCHIVE.md](ARCHIVE.md).

## Quick start

```bash
cd paper/vision && make           # build the vision PDF
cd web && npm install && npm run dev   # the website, http://localhost:3000
cd web && npm run posts           # rebuild blog posts from their Typst sources
```

## License

MIT
