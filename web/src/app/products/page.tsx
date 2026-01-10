import type { Metadata } from "next";
import {
  GameController,
  ChartLineUp,
  Broadcast,
  FloppyDisk,
  ShieldCheck,
  Check,
  BookOpen,
  ChartBar,
  Wrench,
  Chats,
  Gear,
} from "@phosphor-icons/react/dist/ssr";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card } from "@/components/ui";
import { ConvertKitForm } from "@/components/ui/ConvertKitForm";

export const metadata: Metadata = {
  title: "Products",
  description:
    "BVR1 autonomous sidewalk rover: 4-wheel skid-steer, Jetson Orin NX, modular attachments.",
};

export default function ProductsPage() {
  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content">
          {/* HERO */}
          <Card>
            <pre>
{`Sidewalks cleared by sunrise. 10x cheaper than crews.

`}
              <a href="/images/bvr1.png">
                <img src="/images/bvr1.png" alt="bvr1" />
              </a>
{`

One operator monitors ten rovers from a warm office.
Sidewalks cleared in hours, not days. Works in snow.
Open source hardware and software.

`}
              <strong>Starting at $18,000 per rover.</strong>
{`

`}
              <a className="cta-primary" href="#get-started">
                Get started →
              </a>{" "}
              <a className="cta-secondary" href="#how-it-works">
                How it works
              </a>
            </pre>
          </Card>

          {/* THE PROBLEM */}
          <Card title="THE PROBLEM">
            <pre>
{`Snow removal is expensive, slow, and labor-intensive.

`}
              <strong>Traditional crews:</strong>
{`
  • $25-35/hour per worker + equipment
  • Takes days to clear large properties
  • Worker shortages during storms
  • Inconsistent quality
  • Safety liability

`}
              <strong>The market:</strong>
{`
  • $14 billion spent annually in the US
  • Municipalities, universities, airports, facilities
  • ROI timeline: decades (because labor never gets cheaper)

There has to be a better way.`}
            </pre>
          </Card>

          {/* THE SOLUTION */}
          <Card title="THE SOLUTION">
            <pre>
{`Remote-controlled rovers that do the work while you watch.

`}
              <strong>How it works:</strong>
{`

1. Operator connects to rover via web browser
2. Drives it using an Xbox controller with live 360° video
3. Rover clears sidewalks at 1 m/s (~2 mph)
4. LiDAR stops it automatically if anyone gets close
5. One operator can monitor 10+ rovers simultaneously

`}
              <strong>The economics:</strong>
{`

Traditional crew:       4 workers × $30/hr × 8 hrs = $960/day
Muni rover:            1 operator × $30/hr × 8 hrs = $240/day (10 rovers)
                       ────────────────────────────────────────
Cost per rover:        $24/day vs $240/day
`}
              <strong>Savings:           90% cost reduction</strong>
{`

Payback period: <1 season for most deployments.`}
            </pre>
          </Card>

          {/* BVR1 PRODUCT */}
          <Card title="BVR1: THE ROVER" id="bvr1">
            <div className="rover-gallery">
              <a href="/images/bvr0.png" className="rover-thumb">
                <img src="/images/bvr0.png" alt="BVR0 engineering prototype" />
                <span className="rover-label">
                  bvr0 <em>prototype</em>
                </span>
              </a>
              <a href="/images/bvr1.png" className="rover-thumb rover-featured">
                <img src="/images/bvr1.png" alt="BVR1 production rover" />
                <span className="rover-label">
                  bvr1 <em>production</em>
                </span>
              </a>
            </div>

            <pre>
{`Production-ready autonomous sidewalk rover.
Shipping summer 2026.

`}
              <strong>What you get:</strong>
{`

  ◎ 4-wheel skid-steer platform (600×600mm)
  ◈ Jetson Orin NX compute (30 TOPS AI)
  ◉ Livox Mid-360 LiDAR (safety + mapping)
  ◇ Insta360 X4 camera (360° video)
  ⚡ 48V 40Ah battery (~4-8 hour runtime)
  `}
              <Gear size={12} /> Hot-swappable tool system (auger, spreader, plow)
{`
  `}
              <Broadcast size={12} /> LTE connectivity (works anywhere)
{`
  `}
              <ShieldCheck size={12} /> 1-year warranty
{`

`}
              <strong>Safety features:</strong>
{`

  • LiDAR-based obstacle detection (1.5m safety radius)
  • Automatic E-stop on connection loss (250ms timeout)
  • Watchdog timer (catches software crashes)
  • Rate limiting (prevents dangerous commands)
  • No ML in safety path (pure geometry)

`}
              <strong>Specs:</strong>
{`

  Platform:       600mm × 600mm × 400mm
  Weight:         ~60 kg (132 lbs)
  Speed:          0-1 m/s (0-2.2 mph)
  Battery:        48V 40Ah LiFePO4 (~2 kWh)
  Runtime:        4-8 hours (depends on tool usage)
  Sensors:        LiDAR (360×59° FOV), 360° camera, GPS, IMU
  Control:        LTE (100-250ms latency typical)
  Tools:          CAN bus attachment system

`}
              <strong>Price:         $18,000</strong>
{`
Software:      $300/month (optional, or self-host free)

`}
              <a href="#get-started" className="cta">
                Notify me when available
              </a>
            </pre>
          </Card>

          {/* DEPOT */}
          <Card title="DEPOT: THE BASE STATION" id="depot">
            <pre>
{`Fleet management infrastructure. Control center for your rovers.

`}
              <strong>What you get:</strong>
{`

  `}
              <GameController size={12} /> Web-based operator interface
{`
     • Xbox controller support
     • 360° video streaming
     • Real-time telemetry
     • Multi-rover monitoring

  `}
              <ChartLineUp size={12} /> Grafana dashboards
{`
     • Fleet health monitoring
     • Session recording playback
     • Performance metrics
     • Alert system

  `}
              <Broadcast size={12} /> RTK GPS base station (10" Rack option)
{`
     • Centimeter-accurate positioning
     • Enables autonomous navigation
     • Covers ~10km radius

  `}
              <FloppyDisk size={12} /> Session recording
{`
     • SFTP storage (30-day retention)
     • Rerun format for replay
     • Debugging and analysis

`}
              <strong>Three options:</strong>
{`

Self-Hosted (Free):
  • Run on your own hardware
  • Docker Compose deployment
  • Full control, zero cost
  • `}
              <a href="https://github.com/ecto/muni/tree/main/depot">Setup guide on GitHub</a>
{`

10" Rack ($6,000):
  • Pre-assembled hardware
  • RTK base station included
  • 1TB storage
  • Power + ethernet, ready to go
  • `}
              <strong>Coming 2026</strong>
{`

Managed (TBD/month):
  • Fully hosted solution
  • Cloud storage
  • NTRIP corrections
  • Managed updates
  • `}
              <a href="mailto:info@muni.works?subject=Managed%20Depot">Contact us</a>
{`

`}
              <a href="https://github.com/ecto/muni/tree/main/depot" className="cta">
                Get started (free)
              </a>
            </pre>
          </Card>

          {/* HOW IT WORKS */}
          <Card title="HOW IT WORKS" id="how-it-works">
            <pre>
              <strong>Teleoperation (now):</strong>
{`

  1. Operator opens web browser, connects to rover
  2. Plugs in Xbox controller
  3. Sees live 360° H.265 video (100-250ms latency)
  4. Drives rover like a video game
  5. LiDAR creates safety bubble (auto-stops on obstacles)
  6. Rover streams telemetry to Depot
  7. Session recorded for review

One operator can monitor 10+ rovers:
  • Switch between rovers with keyboard shortcuts
  • Automated tasks run with supervision
  • Intervene only when needed

`}
              <strong>Autonomy (coming):</strong>
{`

  • High-definition maps from teleoperation sessions
  • Localization via LiDAR + RTK GPS
  • Planned paths, human-supervised execution
  • Operator monitors, intervenes if needed
  • Gradual transition: 10% → 50% → 90% autonomous

`}
              <strong>Tools (hot-swappable via CAN bus):</strong>
{`

  ├─ Snow auger (breaks up packed snow)
  ├─ Salt/sand spreader (traction control)
  ├─ Plow blade (light snow clearing)
  └─ Mower deck (lawn maintenance, coming soon)

Each tool has its own MCU. Plug it in, rover detects it.`}
            </pre>
          </Card>

          {/* FLEET PACKAGES */}
          <Card title="FLEET PACKAGES" id="packages">
            <pre>
{`Complete turnkey solutions for organizations.
Hardware + software + training + support.

                Pilot       Small       Medium      Large
────────────────────────────────────────────────────────────
Fleet           2 rovers    10 rovers   25 rovers   50 rovers
Base            10" Rack    10" Rack    Rack ×2     Redundant
Training        Remote 4h   On-site 1d  On-site 2d  On-site 1w
Support         Email       Email+phone Priority    Dedicated
────────────────────────────────────────────────────────────
`}
              <strong>Price           $50,000     $220,000    $500,000    $950,000</strong>
{`

`}
              <strong>What&apos;s included:</strong>
{`

  `}
              <Check size={12} /> Rovers with snow removal tools
{`
  `}
              <Check size={12} /> Depot base station with RTK GPS
{`
  `}
              <Check size={12} /> Software licenses (1 year)
{`
  `}
              <Check size={12} /> Operator training
{`
  `}
              <Check size={12} /> Ongoing support
{`
  `}
              <Check size={12} /> Spare parts kit
{`
  `}
              <Check size={12} /> Warranty coverage
{`

`}
              <strong>ROI example (Medium package):</strong>
{`

  Traditional:   25 workers × $30/hr × 500 hrs/season = $375,000/year
  Muni:          1-3 operators × $30/hr × 500 hrs/season = $45,000/year
                 ──────────────────────────────────────────────────────
  Annual savings: $330,000
  Payback:        1.5 seasons

`}
              <a href="mailto:info@muni.works?subject=Fleet%20inquiry" className="cta">
                Get a quote
              </a>
            </pre>
          </Card>

          {/* BUILD YOUR OWN */}
          <Card title="BUILD YOUR OWN (BVR0)" id="open-source">
            <pre>
{`Everything is open source. Build it yourself for ~$5,000.

`}
              <strong>What you need:</strong>
{`

  • BOM: `}
              <a href="https://github.com/ecto/muni/blob/main/bvr/docs/hardware/bom.md">
                ~$5,000 in parts
              </a>
{`
  • Tools: Basic hand tools, soldering iron
  • Time: ~40 hours assembly
  • Skills: Mechanical assembly, basic electronics

`}
              <strong>What you get:</strong>
{`

  `}
              <Check size={12} /> Full CAD files (STEP, STL)
{`
  `}
              <Check size={12} /> Complete schematics and PCB designs
{`
  `}
              <Check size={12} /> Firmware source code (Rust)
{`
  `}
              <Check size={12} /> Assembly manual and build guide
{`
  `}
              <Check size={12} /> Active community support
{`

`}
              <strong>Resources:</strong>
{`

  `}
              <BookOpen size={12} />{" "}
              <a href="/docs/bvr0-manual.pdf">BVR0 Assembly Manual</a> (PDF)
{`
  `}
              <ChartBar size={12} />{" "}
              <a href="/docs/bvr0-datasheet.pdf">BVR0 Datasheet</a> (PDF)
{`
  `}
              <Wrench size={12} />{" "}
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">Build Guide</a>{" "}
              (GitHub)
{`
  `}
              <Chats size={12} />{" "}
              <a href="https://github.com/ecto/muni/discussions">Community Forums</a>
{`

BVR0 is the engineering prototype. Great for R&D, tinkering,
and learning. For production deployments, we recommend BVR1.

`}
              <strong>Warranty: 90 days parts only (DIY builds)</strong>
{`

`}
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware" className="cta">
                Start building
              </a>
            </pre>
          </Card>

          {/* FAQ */}
          <Card title="COMMON QUESTIONS">
            <pre>
              <strong>Is this fully autonomous?</strong>
{`
Not yet. BVR is teleoperated now, with autonomy coming later.
An operator drives it remotely via Xbox controller. The LiDAR
provides automatic safety (stops before hitting things), but
a human is always monitoring.

`}
              <strong>Why not autonomous from day one?</strong>
{`
Safety and trust. We want operators to drive it manually first,
build high-definition maps, understand the edge cases. Then we
layer in autonomy gradually. Plus, teleop is useful even with
autonomy (for edge cases, new areas, tool changes).

`}
              <strong>What&apos;s the video latency like?</strong>
{`
Typically 100-250ms end-to-end. If latency exceeds 500ms, the
system warns the operator and reduces max speed. If connection
drops for 250ms, rover coasts to a safe stop.

`}
              <strong>How does safety work?</strong>
{`
The Livox Mid-360 LiDAR scans 360° × 59° FOV at 200k points/sec.
If anything enters the 1.5m safety radius, immediate E-stop.
This is pure geometry, no ML. At 1 m/s, it stops within 22cm
of detection. No machine learning in the safety path.

`}
              <strong>Can it operate in snow?</strong>
{`
Yes, that's what it's built for. The Livox uses 905nm wavelength
which handles light to moderate snow. Heavy snow reduces range,
but the rover slows down automatically. The LiDAR measures the
snow surface, not what's underneath.

`}
              <strong>How long does the battery last?</strong>
{`
BVR1: 4-8 hours depending on tool usage and terrain. Quick-swap
battery packs enable extended operation. BVR0: 2-3 hours (smaller
battery for prototyping).

`}
              <strong>What tools are available?</strong>
{`
  • Snow auger (breaks up packed snow)
  • Salt/sand spreader (traction control)
  • Plow blade (light clearing)
  • Mower deck (coming soon)

All tools hot-swap via CAN bus. Each has its own MCU that
announces itself when connected.

`}
              <strong>Is everything really open source?</strong>
{`
Yes. All firmware, CAD files, schematics, and documentation
are on `}
              <a href="https://github.com/ecto/muni">GitHub</a>
{` under permissive licenses (MIT, Apache 2.0).
You can build your own BVR0 for ~$5,000. We make money selling
pre-assembled rovers (BVR1) and support contracts.

`}
              <strong>What&apos;s the difference between BVR0 and BVR1?</strong>
{`
BVR0: Engineering prototype, hand-built, ~40h assembly, $5k parts.
       For R&D and learning. Available now (DIY).

BVR1: Production rover, pre-assembled, ~8h final config, $18k.
       For deployments. Shipping summer 2026.

`}
              <strong>Do I need RTK GPS?</strong>
{`
Not for basic teleoperation. RTK provides cm-accurate positioning
for mapping and future autonomy. Recommended but not required.

`}
              <strong>What&apos;s the total cost of ownership?</strong>
{`
Hardware:       $18,000 (one-time)
Software:       $300/month (optional, can self-host free)
LTE data:       $30-50/month per rover
Electricity:    ~$0.50/day per rover
Maintenance:    ~$500/year per rover
Operator time:  1 operator per 10+ rovers

Compare to: $25-35/hour per crew member (traditional).

`}
              <strong>What&apos;s the warranty?</strong>
{`
BVR1: 1 year parts and labor.
BVR0: 90 days parts only (DIY builds).
Software updates are free forever (open source).

`}
              <strong>When can I get one?</strong>
{`
BVR0: Build it now (`}
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">
                instructions on GitHub
              </a>
{`)
BVR1: Preorders coming soon, shipping summer 2026
Fleet packages: `}
              <a href="mailto:info@muni.works?subject=Fleet%20inquiry">Contact us</a>
{` for custom quotes`}
            </pre>
          </Card>

          {/* GET STARTED */}
          <Card title="GET STARTED" id="get-started">
            <pre>
              <strong>Preorders coming soon</strong>
{`

We're finalizing production details and will open preorders
in the coming weeks. Reserve your spot for $500 (fully
refundable until production begins).

Limited to 100 units in the first batch.

`}
              <strong>Want to be notified?</strong>
            </pre>

            <ConvertKitForm />

            <pre>
{`
`}
              <strong>In the meantime:</strong>
{`

  • `}
              <a href="https://github.com/ecto/muni">Explore the source code</a>
{`
  • `}
              <a href="/docs/bvr0-manual.pdf">Read the BVR0 manual</a>
{`
  • `}
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">
                Build your own BVR0
              </a>
{`
  • `}
              <a href="mailto:info@muni.works?subject=Pilot%20program">Join the pilot program</a>
{`

Questions? `}
              <a href="mailto:info@muni.works">info@muni.works</a>
            </pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
