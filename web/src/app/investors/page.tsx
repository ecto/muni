import type { Metadata } from "next";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";

export const metadata: Metadata = {
  title: "Investors",
  description:
    "$500-600k pre-seed for autonomous sidewalk maintenance. $14B market, 10x cost reduction.",
};

export default function InvestorsPage() {
  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content investor-content">
          <Card title="THE PROBLEM">
            <div className="problem-images">
              <figure className="problem-image">
                <img src="/images/slip-fall.jpg" alt="Person slipping on icy sidewalk" />
                <figcaption>1M+ injuries/year</figcaption>
              </figure>
              <figure className="problem-image">
                <img
                  src="/images/pedestrian-road.jpg"
                  alt="Pedestrian forced to walk in road"
                />
                <figcaption>Forced into traffic</figcaption>
              </figure>
            </div>
            <div className="problem-stats">
              <div className="problem-stat">
                <span className="problem-value">1M+</span>
                <span className="problem-label">
                  slip-and-fall injuries per year on icy sidewalks
                </span>
              </div>
              <div className="problem-stat">
                <span className="problem-value">$35B</span>
                <span className="problem-label">municipal liability and clearing costs</span>
              </div>
            </div>
            <p>
              Cities are legally required to clear sidewalks but lack the labor. Manual crews
              are expensive, unreliable, and can&apos;t respond fast enough. Property owners
              ignore ordinances. People get hurt. Cities get sued.
            </p>
          </Card>

          <Card title="THE SOLUTION">
            <div className="solution-images">
              <figure className="solution-image-fig">
                <img src="/images/bvr1.png" alt="BVR1 production rover" />
                <figcaption>bvr1 production</figcaption>
              </figure>
              <figure className="solution-image-fig">
                <img
                  src="/images/bvr0-disassembled.jpg"
                  alt="BVR0 disassembled showing modular design"
                />
                <figcaption>Field-repairable</figcaption>
              </figure>
            </div>
            <div className="solution-points">
              <p>
                <strong>Zero labor cost.</strong> Fully autonomous operation.
                Sidewalks cleared in hours, not days.
              </p>
              <p>
                <strong>Sidewalk-scale.</strong> Modular tools for year-round use: snow
                clearing in winter, mowing in summer.
              </p>
              <p>
                <strong>Open source.</strong> Geometry-based safety (no ML in the critical
                path). Field-repairable with off-the-shelf parts.
              </p>
            </div>
          </Card>

          <Card title="WHY IT WORKS">
            <div className="hero-stat">
              <span className="hero-value">&lt;1 season</span>
              <span className="hero-label">payback for customers</span>
            </div>
            <p className="hero-explain">
              An $18k rover replaces $50k+/year in labor costs. Zero ongoing labor.
              The math is obvious.
            </p>
            <div className="economics-grid">
              <div className="econ-item">
                <span className="econ-value">24/7</span>
                <span className="econ-label">operation</span>
              </div>
              <div className="econ-item">
                <span className="econ-value">$36k</span>
                <span className="econ-label">5-year LTV</span>
              </div>
              <div className="econ-item">
                <span className="econ-value">65%</span>
                <span className="econ-label">hardware margin</span>
              </div>
              <div className="econ-item">
                <span className="econ-value">$300/mo</span>
                <span className="econ-label">recurring software</span>
              </div>
            </div>
          </Card>

          <Card title="MARKET">
            <div className="hero-stat">
              <span className="hero-value">$14B+</span>
              <span className="hero-label">total addressable market</span>
            </div>
            <div className="market-wedge">
              <div className="wedge-item wedge-now">
                <span className="wedge-label">Now</span>
                <span className="wedge-segment">Municipal sidewalks</span>
                <span className="wedge-detail">60M+ miles, highest liability</span>
              </div>
              <div className="wedge-item wedge-next">
                <span className="wedge-label">Next</span>
                <span className="wedge-segment">Universities + Commercial</span>
                <span className="wedge-detail">4,000+ campuses, office parks, HOAs</span>
              </div>
              <div className="wedge-item wedge-future">
                <span className="wedge-label">Then</span>
                <span className="wedge-segment">Year-round platform</span>
                <span className="wedge-detail">Mowing, leaf clearing, line painting</span>
              </div>
            </div>
          </Card>

          <Card title="TRACTION">
            <Pre>
{`2025-12  `}
              <span className="status-complete">■</span>
{` bvr0 engineering prototype complete
         │   Base platform proving out architecture
         │
2026-01  `}
              <span className="status-dev">◐</span>
{` F.Inc Artifact program (SF)
         │   bvr1 R&D, supervised autonomy, 1 production unit
         │
2026-Q3  ○ bvr1 shipping to pilot partners
         │   10 units, Midwest municipalities
         │
2027     ○ Fleet autonomy at scale
             Multi-unit deployments validated`}
            </Pre>
          </Card>

          <Card title="TEAM">
            <div className="team-grid">
              <img src="/images/cam.png" alt="Cam Pedersen" className="team-photo" />
              <div className="team-bio">
                <strong>Cam Pedersen, Founder</strong>
                <p>
                  Autonomous vehicle scheduling at Uber. CTO and co-founder at DitchCarbon.
                  Built this rover from scratch in Cleveland.
                </p>
                <p>
                  Why Cleveland? Real winters. Real municipalities. Lower burn rate than SF. We
                  go where the problem is.
                </p>
                <p>
                  <a href="mailto:info@muni.works" target="_blank" rel="noopener noreferrer">
                    info@muni.works
                  </a>{" "}
                  <a
                    href="https://www.linkedin.com/in/cam-pedersen/"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    LinkedIn
                  </a>{" "}
                  <a href="https://github.com/ecto" target="_blank" rel="noopener noreferrer">
                    GitHub
                  </a>{" "}
                  <a
                    href="https://www.x.com/campedersen"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    X
                  </a>{" "}
                  <a
                    href="https://www.youtube.com/@cam_pedersen/"
                    target="_blank"
                    rel="noopener noreferrer"
                  >
                    YouTube
                  </a>
                </p>
              </div>
            </div>
          </Card>

          <Card title="THE ASK" highlight>
            <div className="ask-box">
              <div className="ask-amount">$500-600K Pre-Seed</div>
              <div className="ask-terms">$3M post-money SAFE</div>
              <div className="ask-use">
                <strong>Use of funds:</strong>
                <ul>
                  <li>Build 10 bvr1 production units</li>
                  <li>Deploy to 3-5 Midwest pilot partners</li>
                  <li>Validate autonomous fleet operations</li>
                  <li>Generate revenue and LOIs for Series A</li>
                </ul>
              </div>
            </div>
          </Card>

          <section className="investor-cta">
            <a href="/docs/pitch-deck.pdf" className="cta-button-large">
              Download Pitch Deck (PDF)
            </a>
            <a href="/docs/one-pager.pdf" className="cta-button-secondary">
              One-Pager
            </a>
            <a href="/docs/whitepaper.pdf" className="cta-button-secondary">
              Technical Paper
            </a>
            <p className="cta-contact">
              Ready to talk?{" "}
              <a href="mailto:info@muni.works?subject=Investment%20Inquiry">info@muni.works</a>
            </p>
          </section>
        </main>

        <Footer />
      </div>
    </div>
  );
}
