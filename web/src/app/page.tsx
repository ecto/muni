import {
  GithubLogo,
  BookOpenText,
  Shield,
  Lightning,
  ArrowRight,
  Eye,
  Crosshair,
  MapPin,
  GameController,
  Cpu,
} from "@phosphor-icons/react/dist/ssr";
import { Footer, FloatingHeader } from "@/components/layout";
import { PlatformViewer } from "@/components/home/PlatformViewer";
import { LidarViewer } from "@/components/home/LidarViewer";
import { LandingRevealProvider } from "@/components/home/LandingRevealProvider";
import { AnimatedMetric } from "@/components/home/AnimatedMetric";
import { HeroRover } from "@/components/home/HeroRover";
import { HeroTerminal } from "@/components/home/HeroTerminal";

export default function HomePage() {
  return (
    <>
      <FloatingHeader />
      <main className="landing">
        {/* ═══════════════════════════════════════════════════════════════════
            CARD 1 — Hero
            The robot is real and it works.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="landing-hero">
          <HeroRover />

          <div className="landing-hero-content">
            <div className="landing-hero-text hero-stagger">
              <div className="landing-hero-eyebrow">
                <span className="landing-hero-eyebrow-dot" />
                Pilot program — Summer 2026
              </div>

              <h1 className="landing-headline">
                Sidewalks that
                <span className="landing-headline-accent">clear themselves.</span>
              </h1>

              <p className="landing-subheadline">
                Autonomous sidewalk maintenance for every season.
                No crews. No overtime. No callbacks.
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
                  <GithubLogo size={14} weight="bold" />
                  <span>Open Source</span>
                </a>
                <div className="landing-trust-divider" />
                <div className="landing-trust-item">
                  <Shield size={14} weight="bold" />
                  <span>LiDAR Safety</span>
                </div>
                <div className="landing-trust-divider" />
                <div className="landing-trust-item">
                  <Lightning size={14} weight="bold" />
                  <span>24/7 Autonomous</span>
                </div>
              </div>
            </div>
          </div>

          <HeroTerminal />

          <div className="landing-hero-scroll">
            <div className="landing-hero-scroll-line" />
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            CARD 2 — The Platform
            It handles every season.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="landing-card landing-platform">
          <div className="landing-container">
            <div className="landing-section-header reveal">
              <span className="landing-eyebrow">The Platform</span>
              <h2 className="landing-section-title">
                One rover. Every season.
              </h2>
              <p className="landing-section-desc">
                Swap attachments, not fleets. Snow, debris, pressure wash — same machine.
              </p>
            </div>
          </div>

          <PlatformViewer />
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            CARD 3 — The Math
            The economics are undeniable.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="landing-card landing-math">
          <div className="landing-container">
            <div className="landing-math-hero reveal">
              <div className="landing-math-hero-value">
                <AnimatedMetric end={96} suffix="%" />
              </div>
              <div className="landing-math-hero-label">cost reduction vs. manual crews</div>
            </div>

            <div className="landing-comparison reveal">
              <div className="landing-comparison-row">
                <div className="landing-comparison-label">Manual Crew</div>
                <div className="landing-comparison-bar-wrap">
                  <div className="landing-comparison-bar landing-comparison-bar-full" />
                </div>
                <div className="landing-comparison-value">$960/day</div>
              </div>

              <div className="landing-comparison-row landing-comparison-highlight">
                <div className="landing-comparison-label">Municipal Robotics</div>
                <div className="landing-comparison-bar-wrap">
                  <div className="landing-comparison-bar landing-comparison-bar-small" />
                </div>
                <div className="landing-comparison-value">$38/day</div>
              </div>
            </div>

            <div className="landing-math-metrics reveal">
              <div className="landing-metric">
                <div className="landing-metric-value">
                  <AnimatedMetric end={50} suffix=" mi" />
                </div>
                <div className="landing-metric-label">cleared per night</div>
              </div>
              <div className="landing-metric-divider" />
              <div className="landing-metric">
                <div className="landing-metric-value">
                  <AnimatedMetric end={365} />
                </div>
                <div className="landing-metric-label">days per year</div>
              </div>
              <div className="landing-metric-divider" />
              <div className="landing-metric">
                <div className="landing-metric-value">$18k</div>
                <div className="landing-metric-label">per unit</div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            CARD 4 — How It Works
            It's safe.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="landing-card landing-howitworks">
          <div className="landing-container">
            <div className="landing-section-header reveal">
              <span className="landing-eyebrow">Safety</span>
              <h2 className="landing-section-title">
                Three zones. Zero guesswork.
              </h2>
              <p className="landing-section-desc">
                LiDAR creates hard geometric boundaries. Stop, slow, clear.
              </p>
            </div>

            <div className="landing-howitworks-layout">
              <div className="landing-howitworks-viz reveal">
                <LidarViewer />
              </div>

              <div className="landing-howitworks-bullets reveal-stagger">
                <div className="landing-tech-bullet reveal">
                  <div className="landing-tech-icon">
                    <Eye size={22} weight="bold" />
                  </div>
                  <div>
                    <h3>360° LiDAR</h3>
                    <p>200k pts/sec. Three concentric safety zones with automatic e-stop.</p>
                  </div>
                </div>

                <div className="landing-tech-bullet reveal">
                  <div className="landing-tech-icon">
                    <Crosshair size={22} weight="bold" />
                  </div>
                  <div>
                    <h3>RTK GPS</h3>
                    <p>Centimeter-accurate positioning. Map once, deploy forever.</p>
                  </div>
                </div>

                <div className="landing-tech-bullet reveal">
                  <div className="landing-tech-icon">
                    <MapPin size={22} weight="bold" />
                  </div>
                  <div>
                    <h3>Behavioral Cloning</h3>
                    <p>5k-parameter policy. Trained offline, runs on-device at 30Hz.</p>
                  </div>
                </div>

                <div className="landing-tech-bullet reveal">
                  <div className="landing-tech-icon">
                    <GameController size={22} weight="bold" />
                  </div>
                  <div>
                    <h3>Fleet Teleop</h3>
                    <p>One operator monitors 10+ rovers. 100ms takeover latency.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            CARD 5 — Built in the Open
            Real engineering, not vaporware.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="landing-card landing-open">
          <div className="landing-open-inner reveal">
            <span className="landing-eyebrow">Built in the Open</span>
            <h2 className="landing-section-title">
              Real engineering. Fully open source.
            </h2>
            <p className="landing-section-desc">
              Every line of firmware, every CAD file, every schematic — on GitHub under MIT.
              No vendor lock-in. No black boxes.
            </p>

            <div className="landing-open-pillars">
              <div className="landing-open-pillar">
                <div className="landing-open-pillar-icon">
                  <GithubLogo size={32} weight="bold" />
                </div>
                <h3>Open Source</h3>
                <p>Firmware, CAD, schematics, docs. Build your own for ~$5k.</p>
              </div>

              <div className="landing-open-pillar">
                <div className="landing-open-pillar-icon">
                  <Cpu size={32} weight="bold" />
                </div>
                <h3>Rust + Jetson</h3>
                <p>Production-grade firmware. Orin NX with 30 TOPS edge AI.</p>
              </div>

              <div className="landing-open-pillar">
                <div className="landing-open-pillar-icon">
                  <BookOpenText size={32} weight="bold" />
                </div>
                <h3>Tested in Cleveland</h3>
                <p>Real winters, real sidewalks. Not a lab demo.</p>
              </div>
            </div>

            <a href="https://github.com/ecto/muni" className="landing-open-cta" target="_blank" rel="noopener noreferrer">
              View on GitHub <ArrowRight size={16} weight="bold" />
            </a>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════
            CARD 6 — CTA
            Schedule a call.
            ═══════════════════════════════════════════════════════════════════ */}
        <section className="landing-card landing-card-compact landing-cta">
          <div className="landing-cta-content reveal">
            <h2 className="landing-cta-title">
              Ready to deploy?
            </h2>

            <p className="landing-cta-desc">
              Schedule a 30-minute call to discuss your city&apos;s needs.
            </p>

            <div className="landing-cta-buttons">
              <a href="https://muni.cal.com/cam/30min" className="landing-btn landing-btn-primary landing-btn-large">
                Schedule a Call
                <ArrowRight size={20} weight="bold" />
              </a>
            </div>

            <div className="landing-cta-alt">
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">Build it yourself</a>
              <span className="landing-cta-alt-divider">/</span>
              <a href="/docs/whitepaper.pdf" target="_blank" rel="noopener noreferrer">Read the whitepaper</a>
              <span className="landing-cta-alt-divider">/</span>
              <a href="mailto:info@muni.works?subject=Pilot%20program">Join pilot program</a>
            </div>
          </div>
        </section>

        <footer className="landing-footer">
          <div className="landing-container">
            <Footer />
          </div>
        </footer>

        <LandingRevealProvider />
      </main>
    </>
  );
}
