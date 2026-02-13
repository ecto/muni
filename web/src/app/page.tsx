import {
  GithubLogo,
  BookOpenText,
  Shield,
  Lightning,
  ArrowRight,
  CheckCircle,
  MapPin,
  Eye,
  Crosshair,
  GameController,
  Snowflake,
  Sun,
  Drop,
} from "@phosphor-icons/react/dist/ssr";
import { Footer, FloatingHeader } from "@/components/layout";
import { HeroVideo } from "@/components/home/HeroVideo";
import { PlatformViewer } from "@/components/home/PlatformViewer";
import { LidarViewer } from "@/components/home/LidarViewer";

export default function HomePage() {
  return (
    <>
      <FloatingHeader />
      <main className="landing">
        {/* ═══════════════════════════════════════════════════════════════════════
            1. HERO — Hype reel video bg + all-season copy
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-hero">
          <HeroVideo />
          <div className="landing-hero-overlay" />

          <div className="landing-container">
            <div className="landing-hero-content">
              <div className="landing-hero-text">
                <div className="landing-badge">
                  <span className="landing-badge-dot" />
                  Pilot program — Summer 2026
                </div>

                <h1 className="landing-headline">
                  Autonomous sidewalk maintenance.
                  <span className="landing-headline-accent"> Zero labor cost.</span>
                </h1>

                <p className="landing-subheadline">
                  Snow. Debris. Pressure wash. One fleet handles every season.
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
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            2. THE PLATFORM — Interactive 3D BVR1 with Snow/Sweep/Wash tabs
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-platform">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">The Platform</span>
              <h2 className="landing-section-title">
                One rover. Every season.
              </h2>
              <p className="landing-section-desc">
                Swap attachments, not fleets. The same rover clears snow in January,
                sweeps debris in April, and pressure-washes in August.
              </p>
            </div>

            <PlatformViewer />
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            3. THE MATH — Broadened cost comparison
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-math">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">The Math</span>
              <h2 className="landing-section-title">
                96% cost reduction. Every season.
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

            <div className="landing-math-metrics">
              <div className="landing-metric">
                <div className="landing-metric-value">50 mi</div>
                <div className="landing-metric-label">cleared per night</div>
              </div>
              <div className="landing-metric">
                <div className="landing-metric-value">365</div>
                <div className="landing-metric-label">days per year</div>
              </div>
              <div className="landing-metric">
                <div className="landing-metric-value">96%</div>
                <div className="landing-metric-label">cost reduction</div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            4. HOW IT WORKS — LiDAR safety viz + tech bullets
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-howitworks">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">How It Works</span>
              <h2 className="landing-section-title">
                Three safety zones. Built into the hardware.
              </h2>
              <p className="landing-section-desc">
                LiDAR rings create physical boundaries — stop, slow, and clear.
                Simple geometry that works every time, rain or shine.
              </p>
            </div>

            <div className="landing-howitworks-layout">
              <div className="landing-howitworks-viz">
                <LidarViewer />
              </div>

              <div className="landing-howitworks-bullets">
                <div className="landing-tech-bullet">
                  <div className="landing-tech-icon">
                    <Eye size={20} weight="bold" />
                  </div>
                  <div>
                    <h3>360° LiDAR</h3>
                    <p>Livox Mid-360 scans at 200k pts/sec. Three concentric safety zones with automatic e-stop.</p>
                  </div>
                </div>

                <div className="landing-tech-bullet">
                  <div className="landing-tech-icon">
                    <Crosshair size={20} weight="bold" />
                  </div>
                  <div>
                    <h3>RTK GPS</h3>
                    <p>Centimeter-accurate positioning. Map once, deploy forever. Follows optimal paths automatically.</p>
                  </div>
                </div>

                <div className="landing-tech-bullet">
                  <div className="landing-tech-icon">
                    <MapPin size={20} weight="bold" />
                  </div>
                  <div>
                    <h3>Behavioral Cloning</h3>
                    <p>Learn from human operators. 5k-parameter MLP policy. Trained offline, runs on-device at 30Hz.</p>
                  </div>
                </div>

                <div className="landing-tech-bullet">
                  <div className="landing-tech-icon">
                    <GameController size={20} weight="bold" />
                  </div>
                  <div>
                    <h3>Fleet Teleop</h3>
                    <p>One operator monitors 10+ rovers. Instant takeover via WebSocket. 100ms control latency.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            5. CITIES — Multi-city pain points + coverage
            ═══════════════════════════════════════════════════════════════════════ */}
        <section className="landing-section landing-cities">
          <div className="landing-container">
            <div className="landing-section-header">
              <span className="landing-eyebrow">Cities</span>
              <h2 className="landing-section-title">
                Every city. Every season.
              </h2>
              <p className="landing-section-desc">
                Sidewalk maintenance isn&apos;t a snow-belt problem. It&apos;s a budget problem.
              </p>
            </div>

            <div className="landing-cities-grid">
              <div className="landing-city-card">
                <div className="landing-city-icon">
                  <Snowflake size={24} weight="bold" />
                </div>
                <h3>Snow Belt</h3>
                <p className="landing-city-examples">Cleveland, Minneapolis, Boston, Chicago</p>
                <ul className="landing-city-pains">
                  <li>$14B/yr US snow removal spend</li>
                  <li>3AM callouts, overtime labor</li>
                  <li>Slip-and-fall liability ($1B+ reserves)</li>
                </ul>
              </div>

              <div className="landing-city-card">
                <div className="landing-city-icon">
                  <Sun size={24} weight="bold" />
                </div>
                <h3>Sun Belt</h3>
                <p className="landing-city-examples">Phoenix, Austin, Miami, Las Vegas</p>
                <ul className="landing-city-pains">
                  <li>Sand, gravel, and debris year-round</li>
                  <li>Heat makes manual labor dangerous</li>
                  <li>Tourism districts need daily cleaning</li>
                </ul>
              </div>

              <div className="landing-city-card">
                <div className="landing-city-icon">
                  <Drop size={24} weight="bold" />
                </div>
                <h3>All Cities</h3>
                <p className="landing-city-examples">SF, Portland, NYC, DC</p>
                <ul className="landing-city-pains">
                  <li>ADA sidewalk compliance pressure</li>
                  <li>Chronic DPW staffing shortages</li>
                  <li>Pressure wash + overnight maintenance</li>
                </ul>
              </div>
            </div>
          </div>
        </section>

        {/* ═══════════════════════════════════════════════════════════════════════
            6. TRUST + SPECS + CTA
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
                  Rate limiting prevents dangerous commands. Hard geometric boundaries.
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

        {/* Specs */}
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

        {/* CTA */}
        <section className="landing-section landing-cta">
          <div className="landing-container">
            <div className="landing-cta-content">
              <div className="landing-cta-badge">
                <CheckCircle size={16} weight="bold" />
                Shipping Summer 2026
              </div>

              <h2 className="landing-cta-title">
                Ready to automate sidewalk maintenance?
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

        <footer className="landing-footer">
          <div className="landing-container">
            <Footer />
          </div>
        </footer>
      </main>
    </>
  );
}
