import Link from "next/link";
import {
  GameController,
  ShieldCheck,
  Snowflake,
  BookOpen,
  Cpu,
  Robot,
  CurrencyDollar,
  Handshake,
  Ruler,
  GitBranch,
  Plugs,
  MapPin,
  ArrowDown,
  Play,
} from "@phosphor-icons/react/dist/ssr";
import { Header, NavBar, Footer } from "@/components/layout";
import { ConvertKitForm } from "@/components/ui/ConvertKitForm";
import { HeroViewer } from "@/components/home/HeroViewer";

// Economics comparison data
const economicsData = [
  { label: "Manual crews", value: 1.67, displayValue: "$1.67M", years: 5 },
  { label: "Contractors", value: 0.98, displayValue: "$0.98M", years: 5 },
  { label: "Muni rovers", value: 0.51, displayValue: "$0.51M", years: 5, highlight: true },
];

export default function HomePage() {
  return (
    <main className="home-frames">
      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 1: THE HOOK
          Goal: Arrest attention, establish what this is in 3 seconds
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-hero">
        <div className="frame-content frame-content-centered">
          <div className="hero-visual">
            <HeroViewer />
          </div>
          <div className="hero-message">
            <h1 className="hero-title">
              Clear 50 miles of sidewalk.
              <br />
              <span className="hero-title-accent">From your desk.</span>
            </h1>
            <p className="hero-value-prop">
              Teleoperated snow rovers for municipalities and campuses.
              <strong> 70% cheaper than manual crews.</strong>
            </p>
            <div className="hero-action">
              <Link href="/products#get-started" className="btn-primary btn-large">
                Buy Now — $500 reserves yours
              </Link>
            </div>
          </div>
        </div>
        <div className="frame-scroll-hint">
          <ArrowDown size={24} />
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 2: THE PROBLEM
          Goal: Create urgency by making the pain visceral
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-problem">
        <div className="frame-content frame-content-centered">
          <div className="problem-visual">
            <div className="problem-image-placeholder">
              <img
                src="/images/pedestrian-road.jpg"
                alt="Pedestrian forced to walk in road due to uncleared sidewalk"
                className="problem-image"
              />
            </div>
          </div>
          <div className="problem-message">
            <p className="problem-eyebrow">The problem</p>
            <h2 className="problem-headline">
              Uncleared sidewalks aren&apos;t just inconvenient.
              <br />
              <span className="problem-headline-accent">They&apos;re dangerous.</span>
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
          FRAME 3: THE SOLUTION
          Goal: Show the product in action, make it feel real
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-solution">
        <div className="frame-content frame-content-centered">
          <div className="solution-visual">
            {/* Video player - commented out until we have footage */}
            {/* <VideoPlayer 
                  src="/videos/rover-clearing.mp4"
                  poster="/images/rover-poster.jpg"
                /> */}
            
            {/* Fallback: Static image or GIF */}
            <div className="solution-video-placeholder">
              <img
                src="/images/bvr1.png"
                alt="BVR1 rover clearing sidewalk"
                className="solution-image"
              />
              <div className="solution-play-overlay">
                <Play size={64} weight="fill" />
                <span>Video coming soon</span>
              </div>
            </div>
          </div>
          <div className="solution-message">
            <p className="solution-eyebrow">The solution</p>
            <h2 className="solution-headline">
              One operator. Ten rovers. Browser-based.
            </h2>
            <div className="solution-features">
              <div className="solution-feature">
                <GameController size={24} />
                <span>Xbox controller with live 360° video</span>
              </div>
              <div className="solution-feature">
                <ShieldCheck size={24} />
                <span>LiDAR safety bubble auto-stops before collision</span>
              </div>
              <div className="solution-feature">
                <Snowflake size={24} />
                <span>Works in active snowfall, day or night</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 4: THE ECONOMICS
          Goal: Make the business case undeniable
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-economics">
        <div className="frame-content frame-content-centered">
          <p className="economics-eyebrow">The economics</p>
          <h2 className="economics-headline">
            Pays for itself in <span className="economics-highlight">11 months</span>
          </h2>
          <p className="economics-subtitle">
            5-year total cost of ownership for 50 miles of sidewalk coverage
          </p>
          
          <div className="economics-chart">
            {economicsData.map((item) => (
              <div
                key={item.label}
                className={`economics-bar-row ${item.highlight ? "economics-bar-highlight" : ""}`}
              >
                <span className="economics-bar-label">{item.label}</span>
                <div className="economics-bar-track">
                  <div
                    className="economics-bar-fill"
                    style={{ width: `${(item.value / 1.67) * 100}%` }}
                  />
                </div>
                <span className="economics-bar-value">{item.displayValue}</span>
              </div>
            ))}
          </div>

          <div className="economics-breakdown">
            <div className="economics-breakdown-item">
              <span className="breakdown-value">$18,000</span>
              <span className="breakdown-label">rover cost</span>
            </div>
            <div className="economics-breakdown-item">
              <span className="breakdown-value">1:10</span>
              <span className="breakdown-label">operator ratio</span>
            </div>
            <div className="economics-breakdown-item">
              <span className="breakdown-value">70%</span>
              <span className="breakdown-label">cost reduction</span>
            </div>
          </div>

          <Link href="/products#packages" className="economics-link">
            See detailed fleet packages →
          </Link>
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 5: HOW IT WORKS
          Goal: Demystify the technology, reduce fear
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-how">
        <div className="frame-content frame-content-centered">
          <p className="how-eyebrow">How it works</p>
          <h2 className="how-headline">
            Simple as a video game. Safer than a human crew.
          </h2>
          
          <div className="how-steps">
            <div className="how-step">
              <span className="how-step-number">1</span>
              <h3>Connect from anywhere</h3>
              <p>Open your browser, connect to your fleet. Works over LTE or WiFi. No app install needed.</p>
            </div>
            <div className="how-step">
              <span className="how-step-number">2</span>
              <h3>Drive with a controller</h3>
              <p>Xbox controller with live 360° video. Rover clears at 1 m/s (~2 mph). Every session recorded.</p>
            </div>
            <div className="how-step">
              <span className="how-step-number">3</span>
              <h3>LiDAR keeps it safe</h3>
              <p>360° LiDAR creates a 1.5m safety bubble. Auto-stops before any collision—even if network drops.</p>
            </div>
          </div>

          <div className="how-trust">
            <p className="how-trust-message">
              <strong>Always supervised. Never fully autonomous without your permission.</strong>
              <br />
              We build trust gradually—autonomy increases only after proven reliability.
            </p>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 6: WHY MUNI (scrollable, not 100vh)
          Goal: Handle objections, differentiate
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame-partial frame-why">
        <div className="frame-content frame-content-centered">
          <p className="why-eyebrow">Why Muni</p>
          <h2 className="why-headline">Built different</h2>
          
          <div className="why-grid">
            <div className="why-item">
              <Handshake size={20} />
              <div>
                <strong>Teleoperation-first</strong>
                <span>Build trust gradually. Autonomy increases after proven reliability.</span>
              </div>
            </div>
            <div className="why-item">
              <CurrencyDollar size={20} />
              <div>
                <strong>Service model available</strong>
                <span>Pay for cleared miles, not units. We&apos;re accountable for outcomes.</span>
              </div>
            </div>
            <div className="why-item">
              <Ruler size={20} />
              <div>
                <strong>600mm compact</strong>
                <span>Fits narrow urban sidewalks where larger competitors can&apos;t.</span>
              </div>
            </div>
            <div className="why-item">
              <GitBranch size={20} />
              <div>
                <strong>Fully open source</strong>
                <span>All firmware on GitHub. Build your own for $5,000.</span>
              </div>
            </div>
            <div className="why-item">
              <Plugs size={20} />
              <div>
                <strong>Integration-ready</strong>
                <span>Works with your GIS, work order, and 311 systems.</span>
              </div>
            </div>
            <div className="why-item">
              <MapPin size={20} />
              <div>
                <strong>Real deployments</strong>
                <span>Operating in Cleveland, OH. Not vaporware.</span>
              </div>
            </div>
          </div>
        </div>
      </section>

      {/* ═══════════════════════════════════════════════════════════════════════
          FRAME 7: FINAL CTA
          Goal: Close the sale
          ═══════════════════════════════════════════════════════════════════════ */}
      <section className="frame frame-cta">
        <div className="frame-content frame-content-centered">
          <p className="cta-eyebrow">Limited availability</p>
          <h2 className="cta-headline">First batch: 100 units</h2>
          <p className="cta-subtitle">
            $500 deposit, fully refundable until production begins Q2 2026.
          </p>
          
          <ConvertKitForm />
          
          <div className="cta-trust-signals">
            <span><BookOpen size={16} /> Fully open source</span>
            <span><MapPin size={16} /> Operating in Cleveland, OH</span>
          </div>

          <div className="cta-alternatives">
            <p>Not ready to commit?</p>
            <div className="cta-alt-links">
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">
                Build it yourself
              </a>
              <Link href="/investors">Read the whitepaper</Link>
              <a href="mailto:info@muni.works?subject=Pilot%20program">
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
  );
}
