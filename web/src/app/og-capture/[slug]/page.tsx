import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { coverFor } from "@/components/blog/covers";
import { formatDate, getPost, posts } from "@/lib/posts";
import "@/styles/og.css";

// Hidden capture route for the social cards: one cover at 1200×630 with the card's
// text over it and no site chrome. The cover runs in og mode, so it draws only when
// scripts/build-og.mjs calls window.__ogRender(t). Not linked, not in the sitemap,
// noindex; the cards it renders are committed under public/og/.
export const metadata: Metadata = { robots: { index: false, follow: false } };

interface Card {
  cover?: string;
  kicker: string;
  title: string;
  subtitle: string;
  site?: boolean;
}

const EXTRA: Record<string, Card> = {
  research: {
    cover: "pinned-rows",
    kicker: "RESEARCH · MUNI.WORKS",
    title: "Research",
    subtitle: "Each one a web page and a PDF, from the same source.",
  },
  site: {
    cover: "solar-system",
    kicker: "MUNICIPAL ROBOTICS · MUNI.WORKS",
    title: "Robots that build robots",
    subtitle: "Arms, pairs and the Rose humanoid, each one measured, from a desk in Ohio to a swarm around a star.",
    site: true,
  },
};

function cardFor(slug: string): Card | undefined {
  const post = getPost(slug);
  if (post) return { cover: post.cover, kicker: `RESEARCH · ${formatDate(post.date).toUpperCase()}`, title: post.title, subtitle: post.subtitle };
  return EXTRA[slug];
}

export function generateStaticParams() {
  return [...posts.map((p) => ({ slug: p.slug })), ...Object.keys(EXTRA).map((slug) => ({ slug }))];
}

export default function OgCard({ params }: { params: { slug: string } }) {
  const card = cardFor(params.slug);
  if (!card) notFound();
  const { Art } = coverFor(card.cover);
  return (
    <div className={`og-card${card.site ? " og-site" : ""}`}>
      <Art og />
      <div className="og-scrim" />
      <div className="og-top">
        {/* eslint-disable-next-line @next/next/no-img-element */}
        <img className="og-logo" src="/images/muni-logo-dark.svg" alt="muni" />
        <span className="og-kicker">{card.kicker}</span>
      </div>
      <div className="og-text">
        <h1>{card.title}</h1>
        <p>{card.subtitle}</p>
      </div>
    </div>
  );
}
