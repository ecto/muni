import {
  GithubLogo,
  BookOpenText,
  MapPin,
  Shield,
  Lightning,
  Sun,
  Eye,
  Gear,
  ArrowRight,
  CheckCircle,
  Clock,
  CurrencyDollar,
} from "@phosphor-icons/react/dist/ssr";
import { Footer, FloatingHeader } from "@/components/layout";
import { HeroViewer } from "@/components/home/HeroViewer";
import { CoverageMapViewer } from "@/components/home/CoverageMapViewer";

export default function HomePage() {
  return (
    <>
      <FloatingHeader />
      <main className="landing">
        {/* ═══════════════════════════════════════════════════════════════════════
            HERO: The Hook
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-hero">
          <div className="landing-hero-bg">
            <div className="landing-hero-grid" />
            <div className="landing-hero-glow" />
          </div>

          <div className="landing-container">
            <div className="landing-hero-content">
              <div className="landing-hero-text">
                <div className="landing-badge">
                  <span className="landing-badge-dot" />
                  Only 2 pilot slots remaining for Winter 2026
                </div>

                <h1 className="landing-headline">
                  Autonomous sidewalk clearing.
                  <span className="landing-headline-accent"> Zero labor cost.</span>
                </h1>

                <p className="landing-subheadline">
                  Deploy rovers that clear 50 miles of sidewalk per night.
                  No crews. No overtime. No 3AM callouts.
                </p>

                <div className="landing-hero-cta">
                  <a href="https://muni.cal.com/cam/30min" className="landing-btn landing-btn-primary">
                    Schedule a Call
                    <ArrowRight size={18} weight="bold" />
                  </a>
                  <a href="/docs/whitepaper.pdf" className="landing-btn landing-btn-secondary" target="_blank" rel="noopener noreferrer">
                    Read Whitepaper
                  </a>
                </div>

                <div className="landing-trust-row">
                  <a href="https://github.com/ecto/muni" className="landing-trust-item" target="_blank" rel="noopener noreferrer">
                    <GithubLogo size={16} weight="bold" />
                    <span>Open Source</span>
                  </a>
                  <div className="landing-trust-divider" />
                  <div className="landing-trust-item">
                    <Shield size={16} weight="bold" />
                    <span>LiDAR Safety System</span>
                  </div>
                  <div className="landing-trust-divider" />
                  <div className="landing-trust-item">
                    <Lightning size={16} weight="bold" />
                    <span>24/7 Operation</span>
                  </div>
                </div>
              </div>

              <div className="landing-hero-visual">
                <div className="landing-hero-viewer-wrap">
                  <HeroViewer />
                  <div className="landing-hero-viewer-label">
                    <span>BVR1</span>
                    <span className="landing-hero-viewer-sublabel">Production Model</span>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            PROBLEM: The Pain
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-problem">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">The Problem</span>
              <h2 className="landing-section-title">
                Snow removal is broken.
              </h2>
              <p className="landing-section-desc">
                Municipalities spend millions on manual labor while pedestrians face dangerous conditions.
              </p>
            </div>

            <div className="landing-stats-grid">
              <div className="landing-stat-card">
                <div className="landing-stat-icon">
                  <CurrencyDollar size={24} weight="bold" />
                </div>
                <div className="landing-stat-value">$14B</div>
                <div className="landing-stat-label">spent annually on snow removal in the US</div>
              </div>

              <div className="landing-stat-card">
                <div className="landing-stat-icon">
                  <Clock size={24} weight="bold" />
                </div>
                <div className="landing-stat-value">$960</div>
                <div className="landing-stat-label">per day for a 4-person manual crew</div>
              </div>

              <div className="landing-stat-card landing-stat-card-highlight">
                <div className="landing-stat-icon">
                  <Shield size={24} weight="bold" />
                </div>
                <div className="landing-stat-value">$1B+</div>
                <div className="landing-stat-label">reserved annually for slip-and-fall liability</div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            SLIP-AND-FALL: The Human Cost
            ═══════════════════════════════════════════════════════════════════════ */}
        <section
          className="landing-section landing-slipfall"
          style={{ backgroundImage: "url('/images/pedestrian-road.jpg')" }}
        >
          <div className="landing-slipfall-overlay" />
          <div className="landing-container">
            <div className="landing-slipfall-content">
              <span className="landing-eyebrow">The Human Cost</span>
              <h2 className="landing-slipfall-title">
                1 million slip-and-fall injuries per year.
              </h2>
              <p className="landing-slipfall-desc">
                Icy sidewalks send people to the emergency room. Lawsuits cost municipalities millions.
                Clear sidewalks save lives and budgets.
              </p>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            COVERAGE: Rovers in Action
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-coverage">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">Coverage</span>
              <h2 className="landing-section-title">
                50 miles cleared per night.
              </h2>
              <p className="landing-section-desc">
                Three rovers working in parallel clear an entire neighborhood before the morning commute.
              </p>
            </div>

            <div className="landing-coverage-viewer">
              <CoverageMapViewer />
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            SOLUTION: The Answer
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-solution">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">The Solution</span>
              <h2 className="landing-section-title">
                The math works.
              </h2>
            </div>

            <div className="landing-comparison">
              <div className="landing-comparison-row">
                <div className="landing-comparison-label">Manual Crew</div>
                <div className="landing-comparison-bar-wrap">
                  <div className="landing-comparison-bar landing-comparison-bar-full" />
                </div>
                <div className="landing-comparison-value">$960/day</div>
              </div>

              <div className="landing-comparison-row landing-comparison-highlight">
                <div className="landing-comparison-label">Muni Rover</div>
                <div className="landing-comparison-bar-wrap">
                  <div className="landing-comparison-bar landing-comparison-bar-small" />
                </div>
                <div className="landing-comparison-value">$38/day</div>
              </div>
            </div>

            <div className="landing-solution-metrics">
              <div className="landing-metric">
                <div className="landing-metric-value">50 mi</div>
                <div className="landing-metric-label">cleared per night</div>
              </div>
              <div className="landing-metric">
                <div className="landing-metric-value">6 AM</div>
                <div className="landing-metric-label">cleared before commute</div>
              </div>
              <div className="landing-metric">
                <div className="landing-metric-value">96%</div>
                <div className="landing-metric-label">cost reduction</div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            FEATURES: How It Works
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-features">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">How It Works</span>
              <h2 className="landing-section-title">
                Fully autonomous operation.
              </h2>
              <p className="landing-section-desc">
                Deploy once. The rover handles the rest.
              </p>
            </div>

            <div className="landing-features-grid">
              <div className="landing-feature-card">
                <div className="landing-feature-icon">
                  <Eye size={28} weight="bold" />
                </div>
                <h3 className="landing-feature-title">360° LiDAR Vision</h3>
                <p className="landing-feature-desc">
                  Livox Mid-360 scans at 200k points/sec. 1.5m safety radius with automatic e-stop.
                  Pure geometry, no ML in the safety path.
                </p>
              </div>

              <div className="landing-feature-card">
                <div className="landing-feature-icon">
                  <MapPin size={28} weight="bold" />
                </div>
                <h3 className="landing-feature-title">RTK GPS Navigation</h3>
                <p className="landing-feature-desc">
                  Centimeter-accurate positioning using HD maps. Define coverage areas once,
                  the rover follows optimal paths automatically.
                </p>
              </div>

              <div className="landing-feature-card">
                <div className="landing-feature-icon">
                  <Sun size={28} weight="bold" />
                </div>
                <h3 className="landing-feature-title">All-Weather Operation</h3>
                <p className="landing-feature-desc">
                  Built for snow. Operates in active snowfall, adapts speed to conditions.
                  4-8 hour runtime with hot-swap battery packs.
                </p>
              </div>

              <div className="landing-feature-card">
                <div className="landing-feature-icon">
                  <Gear size={28} weight="bold" />
                </div>
                <h3 className="landing-feature-title">Modular Attachments</h3>
                <p className="landing-feature-desc">
                  Hot-swap tools via CAN bus. Snow auger, salt spreader, plow blade.
                  Each tool has its own MCU for plug-and-play operation.
                </p>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            TRUST: Why Muni
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-trust">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">Why Muni</span>
              <h2 className="landing-section-title">
                Built different.
              </h2>
            </div>

            <div className="landing-trust-grid">
              <div className="landing-trust-card">
                <div className="landing-trust-card-header">
                  <GithubLogo size={24} weight="bold" />
                  <h3>Fully Open Source</h3>
                </div>
                <p>
                  All firmware, CAD files, schematics, and documentation on GitHub under MIT/Apache 2.0.
                  No vendor lock-in. Build your own for ~$5k.
                </p>
                <a href="https://github.com/ecto/muni" className="landing-trust-link" target="_blank" rel="noopener noreferrer">
                  View on GitHub <ArrowRight size={14} weight="bold" />
                </a>
              </div>

              <div className="landing-trust-card">
                <div className="landing-trust-card-header">
                  <Shield size={24} weight="bold" />
                  <h3>Safety-First Design</h3>
                </div>
                <p>
                  Multi-layer safety: LiDAR detection, watchdog timer, automatic e-stop on connection loss.
                  Rate limiting prevents dangerous commands. No ML in safety path.
                </p>
              </div>

              <div className="landing-trust-card">
                <div className="landing-trust-card-header">
                  <BookOpenText size={24} weight="bold" />
                  <h3>Real Engineering</h3>
                </div>
                <p>
                  Production-grade Rust firmware. Jetson Orin NX compute with 30 TOPS AI.
                  4-wheel skid-steer platform tested in real Cleveland winters.
                </p>
                <a href="/docs/whitepaper.pdf" className="landing-trust-link" target="_blank" rel="noopener noreferrer">
                  Read the whitepaper <ArrowRight size={14} weight="bold" />
                </a>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            SPECS: Quick Reference
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-specs">
          <div className="landing-container">
            <div className="landing-specs-content">
              <div className="landing-specs-header">
                <span className="landing-eyebrow">BVR1 Specifications</span>
                <h2 className="landing-section-title">Production-ready.</h2>
              </div>

              <div className="landing-specs-grid">
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Platform</span>
                  <span className="landing-spec-value">600mm × 600mm × 400mm</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Weight</span>
                  <span className="landing-spec-value">~60 kg (132 lbs)</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Speed</span>
                  <span className="landing-spec-value">0-1 m/s (0-2.2 mph)</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Battery</span>
                  <span className="landing-spec-value">48V 40Ah LiFePO4</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Runtime</span>
                  <span className="landing-spec-value">4-8 hours</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Compute</span>
                  <span className="landing-spec-value">Jetson Orin NX</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">LiDAR</span>
                  <span className="landing-spec-value">Livox Mid-360</span>
                </div>
                <div className="landing-spec-item">
                  <span className="landing-spec-label">Price</span>
                  <span className="landing-spec-value landing-spec-value-highlight">$18,000</span>
                </div>
              </div>

              <a href="/rover" className="landing-specs-link">
                View full specifications <ArrowRight size={14} weight="bold" />
              </a>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            CTA: Close
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-cta">
          <div className="landing-container">
            <div className="landing-cta-content">
              <div className="landing-cta-badge">
                <CheckCircle size={16} weight="bold" />
                Shipping Summer 2026
              </div>

              <h2 className="landing-cta-title">
                Ready to automate your snow removal?
              </h2>

              <p className="landing-cta-desc">
                Schedule a 30-minute call to discuss your deployment needs.
              </p>

              <div className="landing-cta-buttons">
                <a href="https://muni.cal.com/cam/30min" className="landing-btn landing-btn-primary landing-btn-large">
                  Schedule a Call
                  <ArrowRight size={20} weight="bold" />
                </a>
              </div>

              <div className="landing-cta-alt">
                <span>Or explore:</span>
                <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">Build it yourself</a>
                <span className="landing-cta-alt-divider">/</span>
                <a href="mailto:info@muni.works?subject=Pilot%20program">Join pilot program</a>
              </div>
            </div>
          </div>
        </section>

        {/* Footer */}
        <footer className="landing-footer">
          <div className="landing-container">
            <Footer />
          </div>
        </footer>
      </main>
    </>
  );
}
