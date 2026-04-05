import {
  ArrowRight,
  GithubLogo,
} from "@phosphor-icons/react/dist/ssr";
import Link from "next/link";
import { Footer, FloatingHeader } from "@/components/layout";

export default function HomePage() {
  return (
    <>
      <FloatingHeader />
      <main className="landing">
        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 1 — Hero
            Full-bleed video. No text. Let the product speak.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen hero">
          <video
            className="hero-video"
            src="/videos/hype-reel.mp4"
            autoPlay
            muted
            loop
            playsInline
          />
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 2 — The Rover
            Dark CAD render fills the screen. Text anchored bottom-left.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen card card-rover">
          <img
            className="card-bg"
            src="/images/hype-reel-poster.jpg"
            alt=""
            draggable={false}
          />
          <div className="card-scrim" />
          <div className="card-body">
            <h2 className="card-title">The Rover</h2>
            <p className="card-desc">
              Snow, debris, pressure wash — swap the tool, not the fleet.
              Autonomous from dusk to dawn.
            </p>
            <Link href="/rover" className="card-link">
              View specs <ArrowRight size={16} weight="bold" />
            </Link>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 3 — Open Source
            Build flatlay fills the screen. Text anchored bottom-left.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen card card-open">
          <img
            className="card-bg"
            src="/images/bvr0-disassembled.jpg"
            alt=""
            draggable={false}
          />
          <div className="card-scrim" />
          <div className="card-body">
            <h2 className="card-title">Open Source</h2>
            <p className="card-desc">
              Firmware, schematics, mechanical design — MIT licensed on GitHub.
              Build your own or buy from us.
            </p>
            <a href="https://github.com/ecto/muni" className="card-link" target="_blank" rel="noopener noreferrer">
              <GithubLogo size={16} weight="bold" />
              Browse on GitHub <ArrowRight size={16} weight="bold" />
            </a>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 4 — Pilot + CTA
            Pure typography. The ask. Orange accent.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen closer">
          <div className="closer-inner">
            <span className="closer-eyebrow">Pilot Program — Summer 2026</span>
            <h2 className="closer-headline">
              Let&apos;s talk about<br />your sidewalks.
            </h2>
            <p className="closer-desc">
              We&apos;re deploying with 3–5 Midwest cities this summer.
              Early partners shape the product.
            </p>
            <a href="https://muni.cal.com/cam/30min" className="btn btn-primary btn-lg">
              Schedule a Call
              <ArrowRight size={20} weight="bold" />
            </a>
            <div className="closer-alt">
              <a href="/docs/whitepaper.pdf" target="_blank" rel="noopener noreferrer">Read the whitepaper</a>
              <span className="closer-alt-sep">/</span>
              <a href="mailto:info@muni.works?subject=Pilot%20program">Join pilot program</a>
            </div>
          </div>
        </section>

        <footer className="landing-footer">
          <div className="landing-container">
            <Footer />
          </div>
        </footer>
      </main>
    </>
  );
}
