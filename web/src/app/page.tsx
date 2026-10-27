import Link from "next/link";
import { FloatingHeader } from "@/components/layout";
import { EarthriseCover } from "@/components/home/EarthriseCover";
import { glyphs } from "@/components/home/ElementGlyphs";
import { element, links } from "@/lib/plan";
import "@/styles/post.css";
import "@/styles/home.css";

// One frame. What matters now is the first revenue: frog's users becoming pond's customers.
// The plan is one click away; everything else lives in it.

function Tile({ href, external, fam, className, num, stage, name, note, cta, label, glyph: Glyph }: { href: string; external?: boolean; fam: string; className: string; num: string; stage: string; name: string; note: string; cta: string; label: string; glyph?: () => React.JSX.Element }) {
  const body = (
    <>
      <span className="pcard-num">{num}</span>
      {Glyph ? (
        <span className="home-tile-glyph">
          <Glyph />
        </span>
      ) : (
        <span className="pcard-stage">{stage}</span>
      )}
      <span className="pcard-name">{name}</span>
      <span className="pcard-note">{note}</span>
      <span className="home-tile-cta" aria-hidden="true">{cta}</span>
    </>
  );
  const props = { className: `pcard ${className} home-tile`, style: { "--fam": fam } as React.CSSProperties, "aria-label": label };
  return external ? (
    <a {...props} href={href} target="_blank" rel="noopener noreferrer">{body}</a>
  ) : (
    <Link {...props} href={href}>{body}</Link>
  );
}

export default function HomePage() {
  // landing notes are written for a first visit; the paper keeps its own terse ones in elements.json
  // each element in its own neon from the paper's palette; orange stays the one flame (the vision door)
  const cards = [
    { e: element(2), href: links.tang, blurb: "fast inference on any GPU", cta: "tang.toys ↗", color: "#ffd400" },
    { e: element(3), href: links.vcad, blurb: "CAD you can take the derivative of", cta: "vcad.io ↗", color: "#ff2052" },
    { e: element(7), href: links.phyz, blurb: "physics with gradients, even through contact", cta: "phyz.dev ↗", color: "#00f0ff" },
    { e: element(21), href: "https://frog.toys", blurb: "a coding agent that runs on your own machine", cta: "frog.toys ↗", color: "#39ff14" },
  ];
  return (
    <div className="post-page home-page home-one">
      <FloatingHeader />
      <EarthriseCover title="Robots that build robots" subtitle="First, software that writes software. Then arms that build arms.">
        <Link className="home-vision" href="/research/vision">
          <span>Read the vision</span>
          <span className="home-vision-meta" aria-hidden="true">
            desk → shop → road → city → home → moon → mars → star
          </span>
        </Link>
        {/* what exists today, as cards from the paper's periodic table */}
        <div className="post-byline home-cta">
          {cards.map(({ e, href, blurb, cta, color }) => (
            <Tile
              key={e.num}
              href={href}
              external
              fam={color}
              className={`is-${e.maturity}`}
              num={String(e.num)}
              stage={e.stage}
              name={e.name}
              note={blurb}
              cta={cta}
              glyph={glyphs[e.name]}
              label={`${e.name}: ${blurb} (opens ${new URL(href).host})`}
            />
          ))}
        </div>
      </EarthriseCover>
    </div>
  );
}
