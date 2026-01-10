import Link from "next/link";
import {
  GameController,
  ShieldCheck,
  Snowflake,
  BookOpen,
  Check,
} from "@phosphor-icons/react/dist/ssr";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";
import { ConvertKitForm } from "@/components/ui/ConvertKitForm";

export default function HomePage() {
  return (
    <main className="home">
      <div className="container">
        <Header />
        <NavBar />

        {/* HERO */}
        <section className="hero-card">
          <div className="rover-gallery">
            <Link href="/images/bvr0.png" className="rover-thumb">
              <img src="/images/bvr0.png" alt="BVR0 engineering prototype" />
              <span className="rover-label">
                bvr0 <em>prototype</em>
              </span>
            </Link>
            <Link href="/images/bvr1.png" className="rover-thumb rover-featured">
              <img src="/images/bvr1.png" alt="BVR1 production rover" />
              <span className="rover-label">
                bvr1 <em>production</em>
              </span>
            </Link>
          </div>

          <div className="stat-row">
            <div className="stat">
              <span className="stat-value">70%</span>
              <span className="stat-label">cost reduction</span>
            </div>
            <div className="stat">
              <span className="stat-value">1:10</span>
              <span className="stat-label">operator ratio</span>
            </div>
            <div className="stat">
              <span className="stat-value">11mo</span>
              <span className="stat-label">payback</span>
            </div>
          </div>
        </section>

        {/* PITCH */}
        <section className="pitch-card">
          <p className="pitch-headline">
            Clear 50 miles of sidewalk from your office.
          </p>
          <p className="pitch-subtitle">
            Remote-controlled rovers. One operator monitors ten units.
            <br />
            70% cheaper than manual crews. 11-month payback.
          </p>
          <div className="feature-grid">
            <Link href="/products#how-it-works" className="feature">
              <span className="feature-icon">
                <GameController size={14} />
              </span>
              <span className="feature-text">Drive with Xbox controller</span>
            </Link>
            <Link href="/products#bvr1" className="feature">
              <span className="feature-icon">
                <ShieldCheck size={14} />
              </span>
              <span className="feature-text">LiDAR safety system</span>
            </Link>
            <Link href="/products#bvr1" className="feature">
              <span className="feature-icon">
                <Snowflake size={14} />
              </span>
              <span className="feature-text">Works in active snow</span>
            </Link>
            <Link href="/products#open-source" className="feature">
              <span className="feature-icon">
                <BookOpen size={14} />
              </span>
              <span className="feature-text">Fully open source</span>
            </Link>
          </div>
        </section>

        {/* CTA ROW */}
        <section className="cta-row">
          <Link href="/products#get-started" className="cta-primary">
            Reserve yours ($500) →
          </Link>
          <Link href="/products" className="cta-secondary">
            See the economics →
          </Link>
        </section>

        {/* WHY NOW */}
        <Card title="WHY NOW">
          <Pre>
{`Three technologies converged to make this possible:

`}
            <strong>1. Hardware inflection point</strong>
{`
   • E-bike drivetrain: $500 (was $5,000 in 2018)
   • Jetson Orin NX: 100 TOPS for $500 (was $5k+)
   • Livox LiDAR: $1,000 (was $20,000+)
   Complete rover: `}
            <strong>$18,000</strong>
{` (was $100k+ five years ago)

`}
            <strong>2. Proven autonomous navigation</strong>
{`
   • Starship: 12M+ autonomous sidewalk miles
   • Waymo: 96M+ driverless miles, 79% fewer crashes
   • The perception stack exists and works

`}
            <strong>3. Municipal cost crisis</strong>
{`
   • Minneapolis: $40M/year for full municipal clearing
   • 78% of cities use property mandates (70% don't enforce)
   • Labor shortages + injury liability + equity gaps
   • `}
            <strong>No cost-effective alternative has existed. Until now.</strong>
{`

`}
            <Link href="/investors" className="specs-link">
              Read the whitepaper →
            </Link>
          </Pre>
        </Card>

        {/* THE PROBLEM */}
        <Card title="THE PROBLEM">
          <Pre>
{`Uncleared sidewalks kill people and cost millions.

`}
            <strong>Safety:</strong>
{`
  • 65% of pedestrian fatalities occur without clear sidewalks
  • Uncleared paths force pedestrians onto roads
  • Elderly and disabled trapped at home during winter

`}
            <strong>Liability:</strong>
{`
  • $1B reserved annually for slip-and-fall claims
  • Average claim: $19,776
  • 58% of municipalities sued for sidewalk accidents

`}
            <strong>Economics:</strong>
{`
  • Manual crews: $960/day for 4 workers
  • Contractors: Per-event billing, no verification
  • Property mandates: Equity gaps, no enforcement

All three treat this as an episodic labor problem.
`}
            <strong>It&apos;s a continuous coverage problem.</strong>
          </Pre>
        </Card>

        {/* HOW IT WORKS */}
        <Card title="HOW IT WORKS">
          <Pre>
            <strong>Simple as a video game. Safer than a human crew.</strong>
{`

1. Operator opens browser, connects to rover
2. Drives with Xbox controller (live 360° video)
3. LiDAR creates 1.5m safety bubble (auto-stops)
4. Rover clears at 1 m/s (~2 mph)
5. Session recorded for audit trail

`}
            <strong>One operator monitors 10 rovers:</strong>
{`
  • Switch between units with keyboard shortcuts
  • Supervise autonomous segments
  • Intervene only when needed

`}
            <strong>No cloud dependency:</strong>
{`
  • Rovers operate on LTE or local network
  • Local safety systems (work without network)
  • Data syncs when connectivity restored
  • SCADA-like model, proven for decades

`}
            <strong>Progression:</strong>
{`
  Now:    1:1 teleoperation (build reliability data)
  Soon:   1:2 assisted (common routes autopilot)
  Next:   1:10 supervised (operator monitors fleet)
  Future: 1:50+ (full autonomy with human oversight)

Each transition requires one full season of proven reliability.`}
          </Pre>
        </Card>

        {/* THE ECONOMICS */}
        <Card title="THE ECONOMICS">
          <Pre>
            <strong>5-year total cost of ownership (50 miles, 1:10 ratio):</strong>
{`

Manual labor:      $1.67M  (seasonal crews + equipment)
Contractors:       $0.98M  (per-event billing)
`}
            <strong>Muni rovers:       $0.51M  (70% cheaper than manual)</strong>
{`

`}
            <strong>Breakdown:</strong>
{`

  Hardware:        $270,000  (15 rovers @ $18k)
  Software:        $54,000   ($300/mo per rover, or self-host free)
  Operator:        $150,000  (1 operator @ $30k/season × 5 years)
  Maintenance:     $37,500   ($500/year per rover)
                   ────────
  5-year total:    $511,500

`}
            <strong>Payback period: 11 months</strong>
{`

Annual savings vs manual:    $232,000
Annual savings vs contractor: $94,000

`}
            <strong>Not included:</strong>
{`
  `}
            <Check size={12} weight="regular" /> Avoided slip-and-fall liability
{`
  `}
            <Check size={12} weight="regular" /> Eliminated worker injury costs
{`
  `}
            <Check size={12} weight="regular" /> Consistent audit trail (311 complaint defense)
{`
  `}
            <Check size={12} weight="regular" /> Equity (coverage doesn&apos;t depend on property owner ability)
{`

`}
            <Link href="/products#packages" className="specs-link">
              See fleet packages →
            </Link>
          </Pre>
        </Card>

        {/* WHO IT'S FOR */}
        <Card title="WHO IT'S FOR">
          <Pre>
            <strong>Municipalities</strong>
{`
  • 50+ miles of sidewalk network
  • Property mandates failing (inconsistent coverage)
  • Labor shortages during snow events
  • Slip-and-fall liability exposure

`}
            <strong>Universities</strong>
{`
  • Campus pedestrian infrastructure
  • 24/7 operations during snow season
  • Student safety requirements
  • Predictable ROI (multi-year planning)

`}
            <strong>Airports</strong>
{`
  • Outdoor pedestrian paths
  • ADA compliance critical
  • High liability for slip-and-fall
  • Existing autonomous vehicle acceptance

`}
            <strong>Large facilities</strong>
{`
  • Corporate campuses
  • Hospital complexes
  • Industrial parks
  • Distribution centers

`}
            <strong>What they have in common:</strong>
{`
  • Miles of sidewalk to maintain
  • Seasonal labor challenges
  • Audit trail requirements
  • Multi-year budget cycles

`}
            <a href="mailto:info@muni.works?subject=Pilot%20program" className="specs-link">
              Join the pilot program →
            </a>
          </Pre>
        </Card>

        {/* WHY MUNI */}
        <Card title="WHY MUNI">
          <Pre>
            <strong>Teleoperation-first autonomy</strong>
{`
Unlike competitors who promise full autonomy from day one,
we build trust gradually. Operators drive manually first,
creating high-definition maps and understanding edge cases.
Autonomy increases only after proven reliability.

`}
            <strong>Service model, not equipment sales</strong>
{`
Toro sells you an RT-1000. You own the asset and the risk.
We can operate as a managed service: you pay for cleared
miles, not units. We're accountable for outcomes.

`}
            <strong>Compact form factor (600mm)</strong>
{`
Large competitors (RT-1000, Snowbotix) can't fit on narrow
urban sidewalks. ADA minimum width is 36 inches (914mm).
Our 600mm platform navigates where others can't.

`}
            <strong>Open source (no vendor lock-in)</strong>
{`
  • All firmware on GitHub (MIT/Apache 2.0)
  • CAD files, schematics, BOM published
  • Self-host depot software free
  • Build your own for $5,000
  • 15+ year asset lifetime (field-serviceable)

`}
            <strong>Integration-first architecture</strong>
{`
Works with your existing GIS, work order, and 311 systems.
No rip-and-replace. Telemetry feeds your dashboards.

`}
            <strong>Real deployments</strong>
{`
Operating in Cleveland, OH under pilot configuration.
Not vaporware. Specifications reflect actual hardware.

`}
            <Link href="/about" className="specs-link">
              Read the full story →
            </Link>
          </Pre>
        </Card>

        {/* LIMITED AVAILABILITY */}
        <Card title="LIMITED AVAILABILITY" highlight>
          <Pre>
            <strong>First batch: 100 units only</strong>
{`

Preorders opening soon. $500 deposit (fully refundable
until production begins Q2 2026).

Why limit to 100 units?
  • Controlled rollout builds reliability data
  • Direct customer support during first season
  • Iterate on feedback before scaling
  • Prove economics at municipal scale

`}
            <strong>Reserve your spot:</strong>
{`

`}
          </Pre>
          <ConvertKitForm />
          <Pre>
{`
`}
            <strong>Not ready to buy?</strong>
{`
  • `}
            <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">
              Build BVR0 yourself
            </a>
{` (~$5,000 in parts)
  • `}
            <Link href="/investors">Read the whitepaper</Link>
{` (full technical details)
  • `}
            <a href="mailto:info@muni.works?subject=Pilot%20program">
              Join the pilot program
            </a>
{` (test before you buy)

Questions? `}
            <a href="mailto:info@muni.works">info@muni.works</a>
          </Pre>
        </Card>

        <Footer />
      </div>
    </main>
  );
}
