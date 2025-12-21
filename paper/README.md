# Muni Whitepaper

IEEE-style technical paper for municipal robotics procurement.

## Files

- `main.typ` — Typst source using [charged-ieee](https://typst.app/universe/package/charged-ieee) template
- `refs.bib` — BibTeX bibliography
- `main.pdf` — Compiled output

## Diagrams

Uses [CeTZ](https://typst.app/universe/package/cetz) for technical diagrams:

- **Figure 1** — System architecture (operator-rover control loop)
- **Figure 2** — Rover state machine (safety states)
- **Figure 3** — Operator scaling visualization (1:1 vs 1:10)

## Building

### Online (Typst App)

1. Go to [typst.app](https://typst.app)
2. Create new project from template: `charged-ieee`
3. Replace content with `main.typ`
4. Upload `refs.bib`

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

## Template

Uses [charged-ieee](https://typst.app/universe/package/charged-ieee) — IEEE two-column format for engineering papers.

## Structure

1. **Introduction** — Audience and scope
2. **Problem Definition** — Public works as control system, Lakewood reference case
3. **Safety and Liability** — Failure modes, pedestrian interaction, regulatory status
4. **Deployment and Integration** — Pilot sizing, integration, training
5. **Economics** — Baseline costs, capital costs, TCO comparison, sensitivity analysis
6. **Governance** — Data ownership, vendor risk, exit strategy
7. **Conclusion**
8. **Appendix A** — Lakewood, Ohio case study with real numbers

## Target Audience

- Municipal public works directors
- City engineers and procurement officers
- University facilities managers
- Risk officers and city attorneys

## Status

Draft 2 — December 2024
