# Muni Whitepaper

Technical whitepaper for municipal robotics procurement.

## Files

- `main.typ` — Typst source (custom single-column layout)
- `refs.bib` — BibTeX bibliography
- `main.pdf` — Compiled output

## Visualizations

Uses [CeTZ](https://typst.app/universe/package/cetz) for technical diagrams:

- **Figure 1** — System architecture (operator-rover control loop)
- **Figure 2** — Rover state machine (safety states)
- **Figure 3** — Operator scaling visualization (1:1 vs 1:10)

Uses [Lilaq](https://typst.app/universe/package/lilaq) for data visualization:

- **TCO bar chart** — 5-year cost comparison
- **Lakewood TCO chart** — Case study cost comparison

Includes mathematical formulas for:

- Cost per mile calculation
- Seasonal cost modeling
- Operator economics (ratio math)
- Payback period
- Fleet reliability (N+2 redundancy)

## Building

### Local (CLI)

```bash
# Install Typst
brew install typst  # macOS
# or: cargo install typst-cli

# Compile
typst compile main.typ

# Watch mode
typst watch main.typ
```

### Publishing to the website

```bash
# Build + copy all PDFs into web/public/docs/
make web-docs

# Build + copy just the investor PDFs
make investor
```

Vercel runs `make -C paper web-docs` automatically during builds (configured in `web/vercel.json`), so production deployments stay in sync even if you forget to run it locally.

## Layout

Custom single-column layout with:

- Title page with rover image
- Table of contents
- Running headers with revision tracking
- Versioned footer (Rev 1.0)
- IEEE-style bibliography

## Structure

1. **Introduction** — Audience and scope
2. **Problem Definition** — Public works as control system, Lakewood reference case
3. **Why Existing Solutions Fail** — Taxonomy of failures, contractors, consumer robotics
4. **Design Principles** — Service reliability, incremental deployment, spatial redundancy
5. **System Architecture** — SCADA model, local-first, no cloud dependency
6. **Autonomy** — What is automated, what is not
7. **Safety and Liability** — Failure modes, pedestrian interaction, regulatory status
8. **Deployment and Integration** — Pilot sizing, integration, training
9. **Economics** — Baseline costs, capital costs, TCO comparison, sensitivity analysis
10. **Governance** — Data ownership, vendor risk, exit strategy
11. **Roadmap** — Software vs hardware improvements, physics constraints, regulation
12. **Path to Full Autonomy** — Technical requirements, liability shift, regulatory path, timeline honesty
13. **Conclusion** — With pilot program CTA
14. **Appendix A** — Lakewood, Ohio case study
15. **Appendix B** — Environmental and operational specifications

## Images

- `images/rover.jpg` — Title page rover render
- `images/sidewalk-snow.jpg` — Problem illustration (uncleared sidewalk)
- `images/pedestrian-road.jpg` — Problem illustration (pedestrian on road)
- `images/slip-fall.jpg` — Liability illustration (slip-and-fall)

## Target Audience

- Municipal public works directors
- City engineers and procurement officers
- University facilities managers
- Risk officers and city attorneys

## Status

Rev 1.2 — December 2025
