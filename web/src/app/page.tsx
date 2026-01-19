import {
  BookOpen,
  MapPin,
  ArrowDown,
} from "@phosphor-icons/react/dist/ssr";
import { Footer, FloatingHeader } from "@/components/layout";
import { HeroViewer } from "@/components/home/HeroViewer";
import { CoverageMapViewer } from "@/components/home/CoverageMapViewer";

export default function HomePage() {
  return (
    <>
      <FloatingHeader />
      <main className="home-frames">
      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 1: THE HOOK
          Goal: Arrest attention, establish what this is in 3 seconds
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-hero">
        <div className="hero-background">
          <HeroViewer />
        </div>
        <div className="frame-content frame-content-centered hero-content-overlay">
          <div className="hero-message">
            <h1 className="hero-title">
              Clear 50 miles of sidewalk. <span className="hero-title-accent">Autonomously.</span>
            </h1>
            <p className="hero-value-prop">
              Fully autonomous snow clearing rovers for municipalities and campuses.
              <strong> Zero labor cost. Operates 24/7.</strong>
            </p>
            <div className="hero-action">
              <a href="https://buy.stripe.com/dRm8wH3aL91u5mybf3grS00" className="btn-primary btn-large">
                Reserve Yours Now — $999 Deposit
              </a>
            </div>
          </div>
        </div>
        <div className="frame-scroll-hint" aria-hidden="true">
          <ArrowDown size={24} />
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 2: THE PROBLEM
          Goal: Create urgency by making the pain visceral
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-problem">
        <div className="frame-background">
          <img
            src="/images/pedestrian-road.jpg"
            alt=""
            className="frame-background-image"
          />
        </div>
        <div className="frame-content frame-content-centered frame-content-overlay">
          <div className="problem-message">
            <p className="problem-eyebrow">The problem</p>
            <h2 className="problem-headline">
              Uncleared sidewalks aren&apos;t just inconvenient. <span className="problem-headline-accent">They&apos;re dangerous.</span>
            </h2>
            <div className="problem-stats">
              <div className="problem-stat">
                <span className="problem-stat-value">65%</span>
                <span className="problem-stat-label">of pedestrian fatalities occur where sidewalks aren&apos;t cleared</span>
              </div>
              <div className="problem-stat">
                <span className="problem-stat-value">$1B</span>
                <span className="problem-stat-label">reserved annually for slip-and-fall claims</span>
              </div>
              <div className="problem-stat">
                <span className="problem-stat-value">$960</span>
                <span className="problem-stat-label">per day for a 4-person manual crew</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 3: THE PROOF
          Goal: Close the loop - answer the problem stats with solution stats
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-solution">
        <div className="frame-background">
          <CoverageMapViewer />
        </div>
        <div className="frame-content frame-content-centered frame-content-overlay">
          <div className="solution-message">
            <p className="solution-eyebrow">The solution</p>
            <h2 className="solution-headline">
              The math works.
            </h2>
            <div className="solution-stats">
              <div className="solution-stat">
                <span className="solution-stat-value">$38/day</span>
                <span className="solution-stat-label">vs $960 for manual crews</span>
              </div>
              <div className="solution-stat">
                <span className="solution-stat-value">50 mi/night</span>
                <span className="solution-stat-label">per rover</span>
              </div>
              <div className="solution-stat">
                <span className="solution-stat-value">Cleared by 6AM</span>
                <span className="solution-stat-label">before the morning commute</span>
              </div>
            </div>
            <p className="solution-tagline">
              One rover. No overtime. No 3AM callouts.
            </p>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FINAL CTA
          Goal: Close the sale
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-cta">
        <div className="frame-content frame-content-centered">
          <p className="cta-eyebrow">Early access</p>
          <h2 className="cta-headline">First production batch: 100 units</h2>
          <p className="cta-subtitle">
            Reserve your delivery slot. Production begins Q2 2026.
          </p>

          <div className="hero-action">
            <a href="https://buy.stripe.com/dRm8wH3aL91u5mybf3grS00" className="btn-primary btn-large">
              Reserve Yours Now — $999 Deposit
            </a>
          </div>

          <p className="cta-risk-reversal">
            Fully refundable anytime before production begins.
          </p>

          <div className="cta-trust-signals">
            <span><BookOpen size={16} aria-hidden="true" /> Fully open source</span>
            <span><MapPin size={16} aria-hidden="true" /> Operating in Cleveland, OH</span>
          </div>

          <div className="cta-learn-more">
            <p>Learn more:</p>
            <div className="cta-alt-buttons">
              <a href="/docs/whitepaper.pdf" className="cta-button-secondary" target="_blank" rel="noopener noreferrer">Read the whitepaper</a>
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware" className="cta-button-secondary">
                Build it yourself
              </a>
              <a href="mailto:info@muni.works?subject=Pilot%20program" className="cta-button-secondary">
                Join the pilot program
              </a>
            </div>
          </div>
        </div>
      </section>

      {/* Minimal footer */}
      <footer className="frame-footer">
        <div className="frame-content">
          <Footer />
        </div>
      </footer>
    </main>
    </>
  );
}
