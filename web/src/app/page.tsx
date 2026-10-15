import {
  ArrowRight,
  GithubLogo,
} from "@phosphor-icons/react/dist/ssr";
import Link from "next/link";
import { Footer, FloatingHeader } from "@/components/layout";
import { HeroAnimation } from "@/components/home/HeroAnimation";
import { VcadViewer } from "@/components/home/VcadViewer";
import { RoverViewer } from "@/components/home/RoverViewer";
import { ExplodedViewer } from "@/components/home/ExplodedViewer";

export default function HomePage() {
  return (
    <>
      <FloatingHeader />
      <main className="landing">
        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 1 — Hero
            Animated dot-grid canvas. Company identity.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen hero">
          <HeroAnimation />
          <div className="hero-overlay">
            <h1 className="hero-headline">
              Municipal<br />Robotics
            </h1>
            <p className="hero-sub">
              Building robots to help people.
            </p>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 2 — The Rover
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen card card-rover">
          <div className="card-viewer-bg">
            <RoverViewer />
          </div>
          <div className="card-scrim card-scrim-solid" />
          <div className="card-body">
            <h2 className="card-title">The Rover</h2>
            <p className="card-desc">
              Autonomous sidewalk clearing. 50 miles per night, zero labor cost.
              Snow, debris, pressure wash — swap the tool, not the fleet.
            </p>
            <Link href="/rover" className="card-link">
              View specs <ArrowRight size={16} weight="bold" />
            </Link>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 3 — vcad
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen card card-vcad">
          <div className="vcad-viewer-bg">
            <VcadViewer />
          </div>
          <div className="card-scrim card-scrim-solid" />
          <div className="card-body">
            <h2 className="card-title">vcad</h2>
            <p className="card-desc">
              Open-source parametric CAD for the AI era. Modeling, simulation,
              and manufacturing — from sketch to STEP to slicer.
            </p>
            <a href="https://vcad.io" className="card-link" target="_blank" rel="noopener noreferrer">
              Try vcad <ArrowRight size={16} weight="bold" />
            </a>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            SCREEN 4 — Open Source
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="screen card card-open">
          <div className="card-viewer-bg">
            <ExplodedViewer />
          </div>
          <div className="card-scrim card-scrim-solid" />
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
            SCREEN 5 — Closer
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
