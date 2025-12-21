#import "@preview/cetz:0.4.2"
#import "@preview/lilaq:0.5.0" as lq

// Document setup
#set document(
  title: "Robotic Systems for Sidewalk Maintenance",
  author: "Municipal Robotics",
)

#set page(
  paper: "us-letter",
  margin: (x: 1.25in, y: 1in),
  numbering: "1",
  number-align: center,
  header: context {
    if counter(page).get().first() > 1 [
      #set text(size: 9pt, fill: gray)
      Robotic Systems for Sidewalk Maintenance
      #h(1fr)
      Municipal Robotics
    ]
  },
  footer: context {
    if counter(page).get().first() > 1 [
      #set text(size: 8pt, fill: gray)
      Rev 1.0, December 2024
      #h(1fr)
      #counter(page).display()
      #h(1fr)
      Engineering Whitepaper
    ]
  },
)

#set text(font: "New Computer Modern", size: 11pt)
#set par(justify: true, leading: 0.65em)
#set heading(numbering: "1.1")
#show heading.where(level: 1): it => {
  v(1.5em)
  text(size: 16pt, weight: "bold", it)
  v(0.5em)
}
#show heading.where(level: 2): it => {
  v(0.8em)
  text(size: 12pt, weight: "bold", it)
  v(0.3em)
}
#show raw: set text(font: "Menlo", size: 9pt)

// Title page
#align(center)[
  #v(2in)
  #text(size: 24pt, weight: "bold")[
    Robotic Systems \
    for
    Sidewalk Maintenance \
  ]
  #v(0.5em)
  #text(size: 14pt)[
    Why aren't robots shoveling the snow?
  ]
  #v(2em)
  #text(size: 11pt, fill: gray)[
    Technical Whitepaper, December 2024
  ]
  #v(2em)
  #text(size: 12pt)[
    *Municipal Robotics* \
    Cleveland, Ohio \
    #link("mailto:info@muni.works")[info\@muni.works]
  ]
  #v(1em)
  #image("images/rover.jpg", width: 60%)
  #v(1fr)
]

#pagebreak()

// Abstract
#heading(outlined: false, numbering: none)[Abstract]

Municipalities, property managers, and commercial operators maintain hundreds of thousands of miles of sidewalks across the United States. Snow and ice removal on these surfaces is mandated by ADA compliance, tort liability, and ordinance. The labor required is seasonal, episodic, and difficult to staff. Equipment designed for roadways cannot operate on sidewalks. The result is a service gap addressed through overtime, contractors, and deferred maintenance.

This paper describes a robotic system designed to close that gap. The system consists of a small-footprint rover platform (600mm × 600mm) capable of sidewalk navigation, a modular attachment interface supporting snow clearing, sweeping, and brine application, a remote operations model that allows one operator to supervise multiple units, and a fleet coordination layer that integrates with existing municipal GIS and work order systems.

Under supervised autonomy (one operator monitoring ten units), the system reduces five-year total cost of ownership by approximately 70% compared to manual labor and 50% compared to contractors. Even at current 1:1 teleoperation, TCO reduction exceeds 50% versus manual approaches and 20% versus contractors. Additional value from reduced slip-and-fall liability and eliminated worker injuries is not included in these figures. The system is currently deployed in pilot configuration under direct human supervision via LTE teleoperation. Specifications reflect current hardware and software constraints.

#v(1em)
#line(length: 100%, stroke: 0.5pt + gray)
#v(1em)

// Table of contents
#outline(
  title: [Contents],
  indent: 1.5em,
  depth: 2,
)

#pagebreak()

= Introduction

The intended audience for this paper is municipal public works departments, university facilities managers, and commercial property operators evaluating alternatives to manual sidewalk maintenance.

The system described in this paper is operational. Specifications reflect current hardware and software constraints.

@fig:architecture shows the high-level system architecture. The system follows a SCADA-like model: the operator station connects directly to rovers via the local network or LTE, with no cloud dependency. Each rover operates independently with local safety systems that halt the vehicle without network connectivity. Rovers continue autonomous operation during network outages and sync when connectivity is restored.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *

    // Operator station
    rect((-5.5, 0), (-2.5, 2), name: "operator", fill: rgb("#e3f2fd"), stroke: black)
    content("operator.center", align(center)[*Operator*\ Station])

    // Base station / local network
    rect((-1.5, -0.3), (1.5, 2.3), name: "base", fill: rgb("#fff3e0"), stroke: black)
    content((0, 1.8), text(size: 8pt, weight: "bold")[Base Station])
    content((0, 1.2), text(size: 7pt)[Local network])
    content((0, 0.6), text(size: 7pt)[LTE gateway])
    content((0, 0.1), text(size: 7pt)[Data logging])

    // Rover
    rect((2.5, 0), (5.5, 2), name: "rover", fill: rgb("#e8f5e9"), stroke: black)
    content("rover.center", align(center)[*Rover*\ (Jetson + VESC)])

    // Arrows - operator to base
    line((-2.5, 1.3), (-1.5, 1.3), mark: (end: ">"), stroke: 1.5pt)
    line((-1.5, 0.7), (-2.5, 0.7), mark: (end: ">"), stroke: 1.5pt)

    // Arrows - base to rover
    line((1.5, 1.3), (2.5, 1.3), mark: (end: ">"), stroke: 1.5pt)
    line((2.5, 0.7), (1.5, 0.7), mark: (end: ">"), stroke: 1.5pt)

    // Labels
    content((-2, -0.5), text(size: 8pt)[Wired/LAN])
    content((2, -0.5), text(size: 8pt)[LTE / WiFi])

    // Safety callout
    rect((2.7, -1.5), (5.3, -0.3), fill: rgb("#ffebee"), stroke: rgb("#c62828"))
    content((4, -0.9), text(size: 7pt)[Local autonomy:\ No cloud required])
    line((4, -0.3), (4, 0), mark: (end: ">"), stroke: rgb("#c62828") + 1pt)
  }),
  caption: [System architecture: local-first SCADA model with no cloud dependency],
) <fig:architecture>

= Why Now: The Hardware Inflection Point

This system would not have been economically viable five years ago. Several technology trends have converged to create an inflection point for low-cost outdoor robotics.

*48V ecosystem standardization.* The electric bicycle and personal mobility industry has driven massive production scale for 48V lithium-ion batteries, motor controllers, and hub motors. Components that cost \$500+ in 2018 now cost under \$100 at retail. More importantly, this ecosystem has standardized on common form factors, connectors, and protocols. The BVR0 prototype uses an off-the-shelf e-bike battery (\$200), hoverboard hub motors (\$80 each), and VESC motor controllers (\$60 each). Total drivetrain cost: under \$500 for a platform capable of moving 50kg payloads at walking speed.

*Edge compute cost collapse.* The NVIDIA Jetson Orin NX delivers 100 TOPS of AI inference at 15W for under \$500. Five years ago, equivalent compute required \$5,000+ in hardware and 10× the power budget. This enables onboard perception, mapping, and decision-making without cloud connectivity. The Raspberry Pi 5 and similar single-board computers now provide sufficient compute for teleoperation and basic autonomy at \$100.

*Sensor commoditization.* The Livox Mid-360 solid-state LiDAR costs \$1,000 and provides 360° coverage with 40m range. Consumer 360° cameras like the Insta360 X3 (\$400) provide sufficient resolution for remote operation and machine vision. Recent research has demonstrated practical calibration methods for fusing these sensors into coherent spatial representations @bedkowski2025spherical. Five years ago, this sensor suite would have cost \$20,000+.

*Open-source software maturity.* ROS2, OpenCV, PyTorch, and related tools have matured to production quality. Pre-trained models for common perception tasks (pedestrian detection, path segmentation, obstacle classification) are freely available and run efficiently on edge hardware.

The result: a complete sidewalk-clearing robot can be built for under \$5,000 in hardware, using components available from consumer electronics suppliers. This is below the threshold where municipalities can experiment without major capital approval processes.

= Problem Definition: Public Works as a Control System

== The Optimization Problem

Municipal public works departments solve a recurring constrained optimization problem. The objective is to maintain public rights-of-way to a defined service level, subject to fixed annual budgets (typically set 18 months in advance), hard service-level agreements requiring snow clearance within a specified number of hours after snowfall ends, seasonal demand spikes with 10× variance in labor need between summer and winter, an adversarial environment of weather, vandalism, equipment failure, and political pressure, asset lifetime requirements of 15–25 years for equipment, and public accountability where every failure is photographed and posted.

This is a control problem, not a technology problem. The question is not whether robots can clear snow. The question is whether a robotic system can meet service-level guarantees more reliably than the current approach, at equal or lower cost, without introducing new failure modes that the department cannot manage.

The control variables available to a public works director are labor hours allocated per event, fleet size, route sequencing and prioritization, response latency between snowfall end and clearing completion, and equipment availability as a percentage of fleet operational at any given time. Any proposed system must improve at least one of these variables without degrading the others.

== Reference Case: Lakewood, Ohio

Lakewood is a first-ring suburb of Cleveland with a population of 49,517 @census2024lakewood and over 180 miles of sidewalks @lakewood2024sidewalks. It is the most walkable city in Ohio and the state's most densely populated municipality (\~9,000 residents per square mile). The city experiences an average of 24 snow events per season requiring clearing @noaa2024cleveland.

#figure(
  image("images/lakewood-aerial.png", width: 80%),
  caption: [Aerial view of Lakewood, Ohio showing dense residential grid with continuous sidewalk network. Lake Erie and downtown Cleveland visible in background.],
) <fig:lakewood-aerial>

Lakewood presents a compelling case study for several reasons. As a "streetcar suburb" developed in the early 20th century, the city was designed around pedestrian access to transit stops. This legacy produces an unusually complete and well-connected sidewalk network with high daily foot traffic: residents routinely walk to schools, commercial districts, and transit. Sidewalk accessibility is not optional infrastructure; it is the primary mobility layer for a significant portion of the population.

However, this same legacy produces challenges. Aging infrastructure (century-old water mains, overhead power lines, and narrow rights-of-way) creates maintenance complexity. In June 2022, a severe storm system spawned tornadoes that knocked out power across the city for up to two weeks. Cellular connectivity failed within days as tower batteries depleted without grid power. This event demonstrated both the fragility of communications infrastructure and the city's resilience requirements: any deployed system must degrade gracefully when connectivity is unavailable.

Currently, Lakewood does not clear sidewalks municipally. Property owners are required by ordinance to clear adjacent sidewalks within 24 hours of snowfall. Enforcement is handled by the Division of Housing and Building on a complaint basis. The city does not provide school busing, making sidewalk accessibility a student safety issue.

This profile (high density, extensive sidewalk network, heavy pedestrian reliance, aging infrastructure, demonstrated connectivity fragility, property-owner mandate with uneven compliance, and no current municipal clearing budget) represents a common pattern in Midwestern streetcar suburbs and makes Lakewood an ideal testbed for autonomous sidewalk maintenance.

== Current Approaches and Failure Modes

#figure(
  grid(
    columns: 2,
    gutter: 1em,
    image("images/sidewalk-snow.jpg"),
    image("images/pedestrian-road.jpg"),
  ),
  caption: [Uncleared sidewalks force pedestrians onto roads, creating safety hazards and liability exposure],
) <fig:problem>

The consequences are measurable: in 2023, 65% of pedestrian fatalities occurred in locations without a sidewalk or where the sidewalk was obstructed @ghsa2024pedestrian. Sidewalk coverage in major U.S. cities averages only 27–58% of road networks @lee2024sidewalk.

Most municipalities address sidewalk maintenance through one of three approaches:

*1. Municipal crews with hand tools and small equipment*

Typical configuration: seasonal workers with shovels, walk-behind snowblowers, and occasionally ATVs or Toolcats.

Failure modes: Labor availability (snowstorms do not schedule around shift changes), coverage rate (a worker with a shovel clears approximately 0.1 miles per hour), consistency (different workers clear to different standards), and injury (snow removal is among the leading causes of workers' compensation claims in public works @bls2024wages).

*2. Contractor services*

Typical configuration: Landscaping companies with plowing contracts.

Failure modes: Incentive misalignment (per-event contracts reward billing, not coverage), verification (municipalities rarely have real-time visibility into contractor operations), reliability (contractors serve multiple clients), and equipment mismatch (contractors use equipment sized for parking lots).

*3. Property owner mandates*

Typical configuration: Ordinances requiring property owners to clear adjacent sidewalks within N hours.

Failure modes: Enforcement cost, equity (elderly, disabled, and low-income residents cannot comply), and inconsistency (a cleared sidewalk next to an uncleared sidewalk is not a cleared route).

This is the dominant approach. A survey by the Institute of Transportation Engineers found that 78% of municipalities assign sidewalk snow removal responsibility to adjacent property owners @ite2019sidewalk. The legal rationale is liability transfer: if the property owner is responsible, the city is not liable for slip-and-fall injuries.

In practice, enforcement is nearly nonexistent. A University of Delaware study found that 70% of surveyed municipalities did not enforce their sidewalk snow-removal ordinances @udel2010snow. Most cities enforce on a complaint basis only. The result is that *most sidewalks in most American cities are not reliably cleared*. The liability has been transferred on paper, but the service gap remains.

This creates a paradox: cities avoid clearing sidewalks to limit liability, but uncleared sidewalks generate liability anyway. The same study found that 58% of municipalities reported being sued for pedestrian accidents on improperly maintained sidewalks @udel2010snow. Zurich Insurance reserves approximately \$1 billion annually for slip-and-fall claims, with sidewalk incidents averaging \$19,776 per claim @zurich2019slipfall. The current equilibrium is unstable. It persists only because no cost-effective alternative has existed.

== The Structural Problem

All three approaches share a common failure: they treat sidewalk maintenance as an episodic labor problem rather than a continuous coverage problem.

The service requirement is spatial: every linear foot of sidewalk must be cleared. The labor model is temporal: workers clock in and clock out. The mismatch is fundamental.

Heavy equipment solves this mismatch for roadways. A plow truck clears miles per hour. A single operator covers an entire route. But heavy equipment cannot operate on sidewalks. The geometry does not permit it. ADA minimum clear width is 36 inches. A standard plow truck is 102 inches wide.

The result is that sidewalks, the most pedestrian-critical infrastructure, are maintained with the lowest-productivity methods.

== Requirements for a Solution

Any system that claims to address this problem must satisfy the constraints shown in @tab:requirements.

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Requirement*], [*Threshold*], [*Rationale*]),
    [Width], [≤ 30 in (762mm)], [Operate within ADA minimum clear width],
    [Clearing rate], [≥ 0.5 mi/hr], [5× hand labor productivity],
    [Duty cycle], [≥ 4 hrs continuous], [Complete route without returning to base],
    [All-weather], [−20°F to 40°F], [Operate when service is required],
    [Remote operability], [LTE or equivalent], [Supervise from central location],
    [Maintenance], [Field-serviceable], [Repair without factory return],
    [Acquisition cost], [< \$30,000], [Justify against labor savings],
  ),
  caption: [Minimum thresholds for operational viability],
) <tab:requirements>

== What This Paper Does Not Claim

This paper does not claim that robotic sidewalk maintenance is superior to human labor in all circumstances. It claims that robotic systems can extend the coverage capacity of a fixed labor budget, reduce marginal cost per mile at scale, and provide consistent service levels that are difficult to achieve with variable labor.

The system described here is not autonomous in the consumer sense of that word. It requires human operators. It reduces the operator-to-asset ratio, not the operator count to zero.

The system does not eliminate the need for manual crews during extreme weather events. Blizzards, ice storms, and accumulations exceeding the system's clearing capacity (approximately 6 inches per pass) require conventional equipment and personnel. Robotic systems augment baseline capacity; they do not replace surge capacity.

= Why Existing Solutions Fail

This section examines why previous attempts at municipal technology modernization have failed, and what distinguishes viable infrastructure from pilot-stage technology.

== Taxonomy of Municipal Tech Failures

Over the past 15 years, municipal technology pilots have exhibited consistent failure patterns:

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Failure Mode*], [*Example*], [*Root Cause*]),
    [Integration collapse], [Smart city dashboards], [No connection to existing workflows],
    [Vendor dependency], [Proprietary fleet systems], [Lock-in without exit strategy],
    [Scaling cliff], [Autonomous shuttle pilots], [Works at demo scale, fails at city scale],
    [Maintenance gap], [Sensor networks], [No plan for ongoing service],
    [Political discontinuity], [Multi-year IT projects], [Leadership change kills funding],
  ),
  caption: [Common failure modes in municipal technology pilots],
) <tab:failures-taxonomy>

== Why Contractors Underperform

Contractor relationships for sidewalk maintenance fail for structural reasons. Municipalities cannot observe contractor performance in real-time, creating verification asymmetry where quality is measured by complaint volume rather than coverage data. Per-event contracts reward billing frequency while seasonal contracts reward minimal effort per pass, creating incentive misalignment. Contractors serve multiple clients simultaneously, and commercial parking lots pay faster than municipalities. Finally, contractor equipment is sized for parking lots and driveways, not 36-inch sidewalks.

== Why Consumer Robotics Fail in Municipal Applications

Consumer and commercial robots repurposed for municipal use fail on fundamental requirements. Consumer robots expect 1–2 hours of operation while municipal applications require 8+ hour shifts. Consumer IP ratings assume occasional rain, but municipal snow clearing requires operation in active precipitation at −20°F. Consumer products are designed for replacement rather than repair, yet municipal assets must be field-serviceable for 5–15 year lifetimes. Finally, consumer products lack incident logging while municipal operations require full audit trails.

== Why Delivery Robots Don't Transfer

Autonomous delivery robots (Starship, Kiwibot, Serve, Amazon Scout) have logged millions of sidewalk miles. A reasonable question: why not repurpose these platforms for snow clearing?

The answer is that delivery and maintenance are different operational regimes:

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Requirement*], [*Delivery Robot*], [*Maintenance Robot*]),
    [Payload], [Parcels (5–20 kg)], [Snow auger, sweeper (15–30 kg)],
    [Duty cycle], [30–60 min round trips], [4+ hours continuous],
    [Surface contact], [Passive wheels], [Active tool engagement],
    [Operating temp], [Above freezing], [−20°F to 40°F],
    [Weather operation], [Fair weather preferred], [Operates during storms],
    [Motor load], [Light, variable], [Continuous high torque],
    [Maintenance interval], [Depot service], [Field-serviceable daily],
  ),
  caption: [Delivery vs maintenance robot requirements],
) <tab:delivery-vs-maintenance>

Delivery robots optimize for navigation efficiency and payload capacity. Maintenance robots optimize for sustained mechanical work output in adverse conditions. A delivery robot's drivetrain, thermal management, and power system are undersized for snow clearing by factors of 2–5×.

Furthermore, delivery robot business models depend on per-delivery revenue with high utilization. Municipal contracts require guaranteed availability during unpredictable weather events. The operational and economic models are incompatible.

== Why This System Is Different

The system described in this paper is designed around municipal constraints from inception. It is integration-first, designed to connect to existing GIS, work order, and complaint systems rather than replace them. It is operator-centric, keeping human operators in the loop with autonomy increasing incrementally as reliability is demonstrated. It is serviceable, built from commodity components with documented repair procedures. And it is accountable, with full telemetry logging, 90-day retention, and incident replay capability.

= Design Principles

This section describes the engineering principles that guide system design. These principles encode operational constraints that distinguish infrastructure from demonstration technology.

== Service Reliability Over Peak Autonomy

The system is designed to maximize uptime, not autonomy level. A rover that operates reliably at 1:1 teleoperation is more valuable than one that operates autonomously 80% of the time and fails unpredictably 20% of the time.

Autonomy is increased only when reliability at the current level exceeds 95% over a full season.

== Incremental Deployment, Not Citywide Rollouts

Pilot deployments start with 2–3 rovers on 5–15 miles of sidewalk. Expansion occurs only after one full season of validated performance. This approach limits capital risk, allows operational learning, and builds institutional knowledge before scale.

== Human Override as First-Class System

The operator can always take direct control. Override is not an emergency fallback; it is a normal operating mode. The system is designed assuming operators will intervene frequently during early deployment.

== Modular Attachments Instead of Specialized Vehicles

A single rover platform supports multiple attachments:

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Attachment*], [*Season*], [*Function*]),
    [Snow auger], [Winter], [Snow/ice clearing],
    [Brine sprayer], [Winter], [Pre-treatment, de-icing],
    [Rotary sweeper], [Spring/Fall], [Debris, leaves],
    [Inspection camera], [Year-round], [Sidewalk condition assessment],
  ),
  caption: [Modular attachment system],
) <tab:attachments>

This approach reduces capital cost (one platform, multiple uses) and increases utilization (year-round operation).

== Spatial Redundancy Over Mechanical Complexity

Instead of building one highly reliable rover, deploy $N + 2$ rovers for an $N$-rover workload. The probability that at least $N$ rovers are operational is:

$ P_"fleet" = sum_(k=N)^(N+2) binom(N+2, k) p^k (1-p)^(N+2-k) $

where $p$ is single-rover reliability. For $N = 10$ and $p = 0.9$ (90% individual reliability):

$ P_"fleet" approx 0.89 $

The N+2 configuration achieves 89% fleet reliability from 90%-reliable individual units, a significant improvement over 35% reliability with no redundancy ($p^N = 0.9^{10}$). Higher redundancy or improved individual reliability further increases fleet availability.

== Fleet Learning Without Centralized Fragility

Rovers share operational data (route timing, obstacle locations, surface conditions) through the fleet management system. However, each rover can operate independently if network connectivity is lost. There is no single point of failure in the fleet coordination layer.

= System Architecture

This section describes the technical architecture at a level appropriate for IT staff and systems integrators. Detailed specifications are provided in the appendices.

== Platform Overview

#figure(
  table(
    columns: 2,
    align: (left, left),
    table.header([*Component*], [*Specification*]),
    [Dimensions], [600mm × 600mm × 400mm (without attachment)],
    [Weight], [35 kg base platform],
    [Drivetrain], [4-wheel skid-steer, hub motors],
    [Power], [48V Li-ion, 20Ah (960Wh)],
    [Compute], [NVIDIA Jetson Orin NX],
    [Connectivity], [LTE Cat-4 modem],
    [Sensors], [LiDAR (Livox Mid-360), 360° camera],
  ),
  caption: [Platform specifications],
) <tab:platform>

#figure(
  grid(
    columns: 3,
    gutter: 8pt,
    image("images/prototype-pavement.jpg"),
    image("images/prototype-drift.png"),
    image("images/wheel-snow.jpg"),
  ),
  caption: [BVR0 engineering prototype: ultra-low-cost, field-repairable. Pavement testing (left), mid-drift maneuver on grass (center), hoverboard hub motor after snow operation showing acceptable winter traction (right)],
) <fig:bvr0>

#figure(
  grid(
    columns: 2,
    gutter: 12pt,
    image("images/bvr0-disassembled.jpg"),
    image("images/bvr1-render.png"),
  ),
  caption: [Left: BVR0 disassembled for maintenance: aluminum extrusion frame, hoverboard hub motors, e-bike battery, and modular plow attachment. All components replaceable with hand tools in under 30 minutes, with parts generally available from big box stores. Right: BVR1 (rendering), precision-engineered production unit shipping to pilot customers, featuring enclosed weatherproof chassis, integrated plow, RTK GPS, and stereo vision.],
) <fig:bvr0-bvr1>

== Communications Stack

The system uses a layered communications architecture. Transport uses QUIC over UDP for low-latency command and telemetry. Video streams use H.265 RTP at 720p, 30 fps, requiring approximately 2 Mbps. The base station maintains direct connections to rovers via LTE or local WiFi mesh. No cloud services are required for operation. Rovers fail safe on connectivity loss by stopping, holding position, or continuing autonomous waypoint following depending on mode. Typical end-to-end latency from operator input to rover response is 50–150ms over local network, 100–250ms over LTE.

*Safety implications of latency:* At 250ms round-trip latency and maximum speed of 1.5 m/s, a rover travels 375mm before an operator's reaction reaches it. This is well within the 500mm obstacle detection margin. However, latency directly affects operator situational awareness and reaction time. The system compensates by: (1) running obstacle detection locally with zero network dependency, (2) applying velocity limits proportional to latency, and (3) providing latency indicators in the operator UI. If latency exceeds 500ms, the rover automatically reduces speed; above 1000ms, it stops and awaits reconnection.

== Onboard Compute Philosophy

Processing is distributed between edge (rover) and base station. Onboard processing handles real-time, safety-critical functions: the motor control loop at 100 Hz, obstacle detection and emergency stop, watchdog and heartbeat monitoring, telemetry collection, and autonomous waypoint following. The base station handles fleet coordination, dispatch, route optimization, historical data analysis, and incident review. All data remains on-premises. This division ensures rovers operate fully during network outages: they continue clearing assigned routes and sync when connectivity is restored.

== Fleet Coordination Model

The fleet management system provides a dashboard showing real-time status of all rovers including position, battery level, and operational state. Dispatch functions assign routes based on weather conditions and network priority. Automated alerts notify operators of faults, low battery, and connectivity loss. Analytics capabilities generate coverage reports, performance metrics, and cost tracking. The system integrates with municipal GIS via standard formats (Shapefile, GeoJSON) and can export to work order systems via API or file export.

= Autonomy: What Is Automated, What Is Not

This section explicitly separates automated functions from human-controlled functions. This transparency builds trust with operators and regulators.

== Deterministic Behaviors (Fully Automated)

These functions operate without human intervention. Motor control translates velocity commands to wheel speeds. The watchdog stops the rover if no command is received for 250ms. E-stop response immediately halts the rover on command. Low battery response reduces speed and initiates return to base. Obstacle stop halts the rover when LiDAR detects an obstacle within 500mm. These behaviors are implemented in firmware and cannot be overridden by software.

== Learned Perception (Automated with Supervision)

These functions use trained models and require validation. Obstacle classification distinguishes pedestrians, vehicles, and fixed objects. Surface assessment estimates snow depth and ice presence. Path planning selects routes around obstacles. Current status: in development, not deployed in production.

== Human-in-the-Loop Operations (Current)

These functions require human decision-making:

- *Route selection:* Operator assigns rover to route
- *Exception handling:* Operator resolves ambiguous situations
- *Quality verification:* Operator confirms clearing completion
- *Pedestrian interaction:* Operator manages complex encounters

Target state: Reduce operator intervention as autonomy improves, but never eliminate oversight entirely.

== What Is Explicitly Not Automated

The system does not attempt to automate public interaction beyond yielding (no verbal communication or negotiation with pedestrians), property access (will not enter private property or cross driveways autonomously), snow disposal (clears snow to side but does not transport or dump), ice treatment decisions (operator decides when to apply brine), or emergency response (cannot respond to accidents or medical emergencies). These boundaries are intentional. Attempting to automate these functions would increase liability, reduce reliability, and delay deployment.

= Safety and Liability

#figure(
  image("images/slip-fall.jpg", width: 70%),
  caption: [Slip-and-fall incidents on icy sidewalks represent significant municipal liability exposure],
) <fig:liability>

This section addresses safety engineering and liability allocation. It is written for risk officers and city attorneys, not engineers.

== Safety Design Philosophy

The system is designed to fail safe, not fail smart. When uncertainty exceeds thresholds, the rover stops. The priority order is:

+ Do not harm people
+ Do not damage property
+ Do not damage the rover
+ Complete the task

This ordering is enforced in firmware. Task completion is always the lowest priority.

@fig:states shows the rover state machine. The system can only transition to operational states (Teleop, Autonomous) from Idle, and any fault or E-stop immediately halts operations.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *

    // States
    let state-fill = rgb("#e3f2fd")
    let estop-fill = rgb("#ffebee")

    // Disabled
    circle((-3, 2), radius: 0.7, name: "disabled", fill: rgb("#f5f5f5"), stroke: black)
    content("disabled.center", text(size: 8pt)[Disabled])

    // Idle
    circle((0, 2), radius: 0.7, name: "idle", fill: state-fill, stroke: black)
    content("idle.center", text(size: 8pt)[Idle])

    // Teleop
    circle((-1.5, 0), radius: 0.7, name: "teleop", fill: rgb("#e8f5e9"), stroke: black)
    content("teleop.center", text(size: 8pt)[Teleop])

    // Autonomous
    circle((1.5, 0), radius: 0.7, name: "auto", fill: rgb("#e8f5e9"), stroke: black)
    content("auto.center", text(size: 8pt)[Auto])

    // E-Stop
    circle((0, -2), radius: 0.7, name: "estop", fill: estop-fill, stroke: rgb("#c62828") + 1.5pt)
    content("estop.center", text(size: 8pt, fill: rgb("#c62828"))[*E-Stop*])

    // Transitions
    line((-2.3, 2), (-0.7, 2), mark: (end: ">"), stroke: 1pt)
    content((-1.5, 2.4), text(size: 6pt)[Enable])

    line((-0.5, 1.4), (-1.2, 0.6), mark: (end: ">"), stroke: 1pt)
    line((0.5, 1.4), (1.2, 0.6), mark: (end: ">"), stroke: 1pt)

    // Bidirectional between teleop and auto
    line((-0.8, 0), (0.8, 0), mark: (end: ">", start: ">"), stroke: 1pt)

    // To E-Stop (from both operational modes)
    line((-1.2, -0.6), (-0.5, -1.4), mark: (end: ">"), stroke: rgb("#c62828") + 1pt)
    line((1.2, -0.6), (0.5, -1.4), mark: (end: ">"), stroke: rgb("#c62828") + 1pt)

    // E-Stop release back to Idle (curve goes wide right to avoid Auto)
    bezier((0.7, -2), (3.5, -2), (3.5, 2), (0.7, 2), mark: (end: ">"), stroke: 1pt)
    content((3.7, 0), text(size: 6pt)[Release])
  }),
  caption: [Rover state machine: E-stop is reachable from any operational state],
) <fig:states>

== Failure Modes and Responses

@tab:failures shows the system response to various failure conditions.

#figure(
  table(
    columns: 4,
    align: (left, left, left, left),
    table.header([*Condition*], [*Detection*], [*Response*], [*Recovery*]),
    [Obstacle detected], [LiDAR, camera], [Stop, assess, route around], [Automatic or escalate],
    [Communication loss], [Heartbeat timeout], [Coast to stop, hold], [Auto-resume on reconnect],
    [Operator loss], [Heartbeat timeout], [Zero velocity], [Resume when operator returns],
    [Low battery], [Voltage monitor], [Reduce speed, return], [Charge cycle],
    [Critical battery], [Voltage monitor], [Safe stop, disable], [Manual recovery],
    [Hardware fault], [Self-diagnostics], [Safe stop, alert], [Manual inspection],
    [E-stop activated], [Operator command], [Immediate stop], [Explicit release required],
  ),
  caption: [Failure modes and system responses],
) <tab:failures>

== Pedestrian Interaction

The system operates on shared pedestrian infrastructure. Maximum speed is 1.2 m/s in normal operation, reduced to 0.5 m/s when pedestrians are detected within 3 meters. The rover always yields to pedestrians and does not attempt to pass or navigate around people in motion. Stopping distance is less than 500mm at maximum speed on dry pavement and less than 1 meter on snow or ice. Visibility is provided by amber marker lights at all corners and retroreflective markings on all sides, with an optional low-volume alert tone before movement. The system does not rely on pedestrians to behave predictably. If a pedestrian stops in front of the rover, the rover waits indefinitely.

== Incident Logging and Replay

All operational data is logged. Telemetry (position, velocity, motor currents, battery state) is recorded at 1 Hz and retained for 90 days. All operator commands are logged with timestamps and retained for 90 days. Video is recorded continuously during operation and retained for 30 days. Events (obstacles detected, stops triggered, faults occurred) are retained indefinitely. Logs are stored locally on the rover and synced to the base station. In the event of an incident, complete session replay is available within hours.

== Insurance and Liability

*Coverage model:* Robotic sidewalk equipment is classified as mobile equipment under standard commercial general liability (CGL) policies. Most municipal insurers (CIRMA, PennPRIME, OMAG, similar pools) cover robotic operations under existing fleet or equipment endorsements without separate riders.

*Typical coverage structure:*
- General liability: \$1–2M per occurrence (existing municipal policy)
- Equipment floater: Replacement value per unit (\$15–25k)
- Cyber liability: Recommended for fleet management systems
- Umbrella/excess: Per municipal risk tolerance

*Premium impact:* Early deployments report premium increases of \$200–600 per rover annually, comparable to ride-on mowers or utility vehicles. Insurers familiar with autonomous equipment (from warehouse and agricultural robotics) typically require operational documentation and incident response procedures rather than specialized policies.

*Vendor liability:* The vendor warrants that the system performs as specified. The vendor does not assume operational liability for incidents arising from operator error, environmental conditions outside specified limits, or unauthorized modifications.

*Incident investigation:* In the event of an incident involving injury or significant property damage, the vendor will provide full access to telemetry and logs, technical support for investigation, and cooperation with legal and regulatory processes.

== Regulatory Status

Sidewalk robots are not federally regulated in the United States. Regulation, where it exists, is at the state or municipal level. As of December 2024, 14 states have enacted personal delivery device (PDD) legislation @pdd2024legislation with weight limits typically ranging from 80–550 lbs and speed limits of 6–12 mph. Most require yielding to pedestrians, operator oversight, and liability insurance.

The system described in this paper is designed to comply with the most restrictive common requirements.

= Deployment and Integration

This section describes how the system is deployed in practice. It is written for operations managers and IT staff.

== Pilot Sizing

Recommended pilot configuration is shown in @tab:pilot.

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Component*], [*Quantity*], [*Notes*]),
    [Rovers], [2–3], [Allows comparison, provides redundancy],
    [Attachments], [1 per rover], [Match to season],
    [Operator workstation], [1], [Can supervise all pilot units],
    [Charging infrastructure], [1 bay per rover], [Co-located with storage],
  ),
  caption: [Recommended pilot configuration],
) <tab:pilot>

*Pilot duration:* Minimum one full season (3–4 months for snow) to observe performance across weather conditions.

*Pilot scope:* 5–15 miles of sidewalk, selected for mix of conditions, accessible staging location, and representative of broader network.

== Integration Touchpoints

The system integrates with existing municipal infrastructure at several points. GIS and mapping integration imports the sidewalk network as routes using standard formats with low complexity. Work order integration exports clearing logs and can optionally receive dispatch commands at medium complexity. Weather service integration receives forecasts for pre-positioning at low complexity. Citizen complaint systems can cross-reference complaints with clearing logs at medium complexity. Fleet management provides a dashboard for status, telemetry, and alerts. Full integration is not required for pilot; minimum viable integration is GIS import for route planning.

== Training

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Role*], [*Training Time*], [*Content*]),
    [Operator], [4–8 hours], [Teleoperation, monitoring, exception handling, safety],
    [Supervisor], [2–4 hours], [Fleet dashboard, reporting, escalation],
    [Maintenance], [8–16 hours], [Inspection, consumables, repairs, diagnostics],
  ),
  caption: [Training requirements by role],
) <tab:training>

Training is provided on-site during commissioning. Refresher training recommended annually.

== Storage and Maintenance Facility

Requirements: 100 sq ft per rover, 20A 120V circuit per 2 rovers, above-freezing climate preferred, locked facility with GPS tracking and remote disable, internet access for telemetry sync.

Most municipalities can accommodate pilots in existing public works facilities.

= Economics

This section presents the economic case for robotic sidewalk maintenance. All figures are based on current hardware costs, observed productivity rates, and published municipal labor data.

== Baseline: Current Municipal Costs

*Methodology note:* Productivity figures are based on timed clearing of marked sidewalk segments (100m intervals) across four snow events in Northeast Ohio during the 2024–2025 season.

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Parameter*], [*Value*], [*Source*]),
    [Loaded labor rate], [\$35–45/hour], [BLS, municipal contracts],
    [Productivity (shovel)], [0.08–0.12 mi/hr], [Field observation],
    [Productivity (blower)], [0.15–0.25 mi/hr], [Manufacturer spec],
    [Snow events/season], [15–25], [NOAA climate data],
    [Clearing requirement], [4–12 hrs post-snowfall], [Municipal ordinance],
  ),
  caption: [Manual labor cost parameters],
) <tab:labor>

The cost per mile cleared is derived from labor rate and productivity:

$ C_"mile" = L / P $

where $L$ is the loaded labor rate (\$/hour) and $P$ is productivity (miles/hour). At $L = 40$ and $P = 0.12$ (blended shovel and blower work):

$ C_"mile" = 40 / 0.12 approx 333 " dollars/mile" $

For a city with $M = 50$ miles of priority sidewalk network and $E = 20$ events per season:

$ C_"season" = C_"mile" times M times E = 333 times 50 times 20 = 333,000 $

Per-event contractor rates range from \$150–400 per mile. Seasonal contracts for 50 miles of sidewalk typically range from \$150,000–400,000, with \$200/mile being a common benchmark.

== System Capital Costs

#figure(
  table(
    columns: 2,
    align: (left, right),
    table.header([*Category*], [*Cost*]),
    [Chassis and drivetrain], [\$950],
    [Electronics (compute, CAN, LTE)], [\$890],
    [Perception (LiDAR, camera)], [\$1,820],
    [Power system], [\$400],
    [Snow clearing attachment], [\$365],
    [Assembly, wiring, integration], [\$400],
    [*Total hardware cost*], [*\$4,825*],
  ),
  caption: [Per-unit hardware cost (current prototype, ~\$5,000)],
) <tab:bom>

Target production cost at scale (100+ units): *\$3,400*. Sale price (target): \$12,000 base, \$15,000 with snow clearing, \$18,000 with full sensor suite.

*Asset lifetime:* 5-year service life with annual maintenance. Mid-life refurbishment at year 3 (\$800–1,200). Chassis can be refurbished for a second 5-year cycle at approximately 40% of new unit cost.

== Fleet Sizing

The number of rovers required depends on network size, clearing time window, and redundancy requirements.

#figure(
  table(
    columns: 2,
    align: (left, left),
    table.header([*Parameter*], [*Value*]),
    [Clearing rate], [0.5 mi/hr],
    [Battery endurance], [4 hours continuous],
    [Miles per rover per charge], [2 miles],
    [Clearing time window], [8–12 hours post-snowfall],
    [Effective miles per rover per event], [4 miles (with one recharge)],
  ),
  caption: [Rover productivity assumptions],
) <tab:productivity>

For a 50-mile network with 8-hour clearing window:

$ N_"rovers" = M / P_"event" = 50 / 4 = 12.5 arrow.r 13 " rovers" $

With N+2 redundancy for 90%+ fleet availability: *15 rovers*.

Capital investment: $15 times 18,000 = 270,000$

== Operating Costs

Annual per-rover operating costs vary significantly by operator ratio:

#figure(
  table(
    columns: 4,
    align: (left, right, right, right),
    table.header([*Cost Category*], [*1:1 Teleop*], [*1:10 Supervised*], [*Notes*]),
    [Operator labor], [\$4,000], [\$400], [160 hrs/season × \$25/hr ÷ ratio],
    [LTE connectivity], [\$360], [\$360], [Fixed],
    [Maintenance], [\$500], [\$500], [Fixed],
    [Battery (amortized)], [\$200], [\$200], [Fixed],
    [Charging energy], [\$125], [\$125], [Fixed],
    [Software subscription], [\$1,200], [\$1,200], [Fixed],
    [Insurance], [\$400], [\$400], [Fixed],
    [*Total per rover*], [*\$6,785*], [*\$3,185*], [],
  ),
  caption: [Annual operating cost per rover by supervision mode],
) <tab:opcost>

The operator-to-rover ratio is the dominant variable. At 1:10 supervision, operating costs drop by 53% compared to 1:1 teleoperation.

== Operator Economics

The viability of robotic sidewalk maintenance depends on the operator-to-rover ratio $R$. The effective labor cost per rover-hour is:

$ C_"rover-hr" = L_"op" / R $

where $L_"op"$ is the operator hourly rate. At $L_"op" = 25$ and $R = 10$:

$ C_"rover-hr" = 25 / 10 = 2.50 " dollars/rover-hour" $

This represents a 10× reduction in labor cost per unit of work compared to 1:1 teleoperation.

#figure(
  table(
    columns: 4,
    align: (left, left, right, right),
    table.header([*Mode*], [*Ratio*], [*Op. Cost/hr*], [*Cost/Rover-Hr*]),
    [Direct teleop], [1:1], [\$25], [\$25.00],
    [Assisted teleop], [1:2], [\$25], [\$12.50],
    [Supervised autonomy], [1:10], [\$25], [\$2.50],
    [Full autonomy], [1:50+], [\$25], [\$0.50],
  ),
  caption: [Operator economics by autonomy level],
) <tab:autonomy>

@fig:scaling illustrates the operator scaling difference. At 1:1 (current), each rover requires a dedicated operator. At 1:10 (target), one operator monitors ten rovers with autonomous waypoint following.

#figure(
  cetz.canvas(length: 0.8cm, {
    import cetz.draw: *

    // 1:1 Mode (left side)
    content((-4, 3.5), text(weight: "bold", size: 9pt)[1:1 Teleop (Current)])

    // One operator, one rover
    for i in range(3) {
      let y = 2 - i * 1.2
      // Operator icon (circle head + body)
      circle((-5.5, y + 0.3), radius: 0.25, fill: rgb("#1976d2"), stroke: none)
      rect((-5.8, y - 0.3), (-5.2, y + 0.1), fill: rgb("#1976d2"), stroke: none)
      // Arrow
      line((-4.9, y), (-4.1, y), mark: (end: ">"), stroke: 1pt)
      // Rover (rectangle)
      rect((-4, y - 0.3), (-3, y + 0.3), fill: rgb("#4caf50"), stroke: black)
      content((-3.5, y), text(size: 6pt, fill: white)[R#str(i+1)])
    }
    content((-4.2, -1.5), text(size: 8pt)[3 operators\ 3 rovers])

    // Divider
    line((0, 3.5), (0, -1.8), stroke: (dash: "dashed", paint: gray))

    // 1:10 Mode (right side)
    content((4, 3.5), text(weight: "bold", size: 9pt)[1:10 Supervised (Target)])

    // One operator
    circle((2, 1), radius: 0.35, fill: rgb("#1976d2"), stroke: none)
    rect((1.6, 0.2), (2.4, 0.7), fill: rgb("#1976d2"), stroke: none)

    // Fan out to 10 rovers (show 5 for space)
    for i in range(5) {
      let angle = 30 - i * 15
      let x = 4.5 + i * 0.8
      let y = 2.5 - i * 0.6
      line((2.5, 0.8), (x - 0.4, y), mark: (end: ">"), stroke: 0.8pt + rgb("#666"))
      rect((x - 0.35, y - 0.25), (x + 0.35, y + 0.25), fill: rgb("#4caf50"), stroke: black)
      content((x, y), text(size: 5pt, fill: white)[R#str(i+1)])
    }

    // "..." for more rovers
    content((6.5, -0.5), text(size: 10pt)[...])

    content((4.5, -1.5), text(size: 8pt)[1 operator\ 10 rovers])

    // Cost comparison
    rect((-6.5, -2.8), (-2, -2.2), fill: rgb("#ffebee"), stroke: rgb("#c62828"))
    content((-4.25, -2.5), text(size: 7pt)[\$75/hr labor])

    rect((1.5, -2.8), (7, -2.2), fill: rgb("#e8f5e9"), stroke: rgb("#2e7d32"))
    content((4.25, -2.5), text(size: 7pt)[\$25/hr labor (10× efficiency)])
  }),
  caption: [Operator scaling: 1:1 teleop vs 1:10 supervised autonomy],
) <fig:scaling>

*Current capability:* Direct teleoperation (1:1). Operator labor savings come from reduced physical labor and reduced injury risk, not from ratio improvement.

*Target capability:* Supervised autonomy (1:10). This requires autonomous waypoint following, static obstacle detection, dynamic obstacle avoidance, and exception handling. These capabilities are in active development.

*Labor considerations:* Robotic systems change the nature of sidewalk maintenance labor; they do not eliminate it. Operators are typically drawn from existing staff and reassigned from physical clearing to supervisory roles.

== Operator Workload and Ergonomics

At 1:10 supervision ratios, operator fatigue becomes a design constraint. Monitoring ten simultaneous video feeds for 4–8 hours induces cognitive load that differs qualitatively from physical labor fatigue.

*Shift structure:* Recommended maximum shift length is 4 hours of active supervision with 15-minute breaks every 90 minutes. Snow events requiring 8+ hours of clearing should use rotating operator pairs.

*Workstation design:* Operators work from climate-controlled stations with ergonomic seating, multiple monitors (one primary view, one fleet overview), and low-latency audio alerts for exceptions. Physical stress is minimal; cognitive stress requires active management.

*Attention allocation:* At 1:10, operators do not watch all feeds continuously. The system surfaces exceptions (obstacle stops, low battery, connectivity loss, pedestrian encounters) and the operator responds to alerts. Between exceptions, operators cycle through rover views on a 30-second rotation. Autonomous waypoint following handles nominal operation.

*Fatigue indicators:* Response time to alerts, intervention frequency, and override accuracy are logged per operator session. Degradation beyond baseline triggers mandatory breaks or shift handoff.

This operational model mirrors air traffic control and industrial SCADA supervision rather than vehicle operation. Staffing plans should account for the distinct fatigue profile.

== Total Cost of Ownership Comparison

Scenario: 50 miles of priority sidewalk, 20 snow events per season, 5-year analysis period, 15-rover fleet.

#figure(
  table(
    columns: 4,
    align: (left, right, right, right),
    table.header([*Approach*], [*Year 1*], [*Years 2–5*], [*5-Year TCO*]),
    [Manual labor (municipal)], [\$333,000], [\$333,000/yr], [\$1,665,000],
    [Contractor (\$200/mi)], [\$200,000], [\$200,000/yr], [\$1,000,000],
    [Robotic (1:1 teleop)], [\$372,000], [\$102,000/yr], [\$780,000],
    [Robotic (1:10 supervised)], [\$318,000], [\$48,000/yr], [\$510,000],
  ),
  caption: [Total cost of ownership comparison (50 miles, 15 rovers)],
) <tab:tco>

Robotic Year 1 includes capital (\$270,000) plus first-year operating costs. Years 2–5 are operating costs only.

@fig:tco-chart visualizes the 5-year TCO comparison. At supervised autonomy (1:10), robotic systems reduce total cost by *69% vs manual labor* and *49% vs contractors*.

#figure(
  lq.diagram(
    width: 6cm,
    height: 3.5cm,
    xaxis: (ticks: ((0, [Manual]), (1, [Contract]), (2, [1:1]), (3, [1:10]))),
    yaxis: (label: [TCO (\$M)]),
    lq.bar(
      (0, 1, 2, 3),
      (1.67, 1.00, 0.78, 0.51),
      width: 0.6,
      fill: rgb("#4caf50"),
    ),
  ),
  caption: [5-year total cost of ownership comparison (50 miles, 20 events/season)],
) <fig:tco-chart>

*Payback period:* The payback period $T_"payback"$ in months is:

$ T_"payback" = C_"capital" / (C_"manual" - C_"robotic") times 12 $

At 1:10 supervision:

$ T_"payback" = 270000 / (333000 - 48000) times 12 approx 11.4 " months" $

At 1:1 teleoperation, payback extends to approximately 14 months. Both scenarios achieve payback within the first full season of operation.

== Liability and Injury Avoidance

Beyond direct operating costs, robotic systems reduce two categories of indirect cost:

*Slip-and-fall liability:* As noted earlier, 58% of municipalities have been sued for pedestrian accidents on improperly maintained sidewalks @udel2010snow, with average claims of \$19,776 @zurich2019slipfall. Consistent robotic clearing reduces both incident frequency and legal exposure. If a municipality currently experiences 2–5 claims per year (\$40,000–100,000), even a 50% reduction represents \$20,000–50,000 in annual savings, not including legal defense costs.

*Worker injury reduction:* Snow removal ranks among the highest-risk municipal activities for musculoskeletal injuries. A single worker's compensation claim averages \$30,000–50,000. Shifting from physical shoveling to supervisory roles eliminates this exposure for assigned staff. For a crew of 10 seasonal workers, preventing 1–2 claims per season represents \$30,000–100,000 in avoided costs.

These indirect savings are difficult to guarantee but can exceed direct labor savings in high-claim environments. They should be considered qualitatively when evaluating total value.

== Sensitivity Analysis

The economic model is most sensitive to operator ratio and snow event frequency:

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Variable*], [*Base Case*], [*Break-Even vs Contractor*]),
    [Operator ratio], [1:10], [1:3],
    [Snow events/season], [20], [10],
    [Clearing rate], [0.5 mi/hr], [0.25 mi/hr],
    [Hardware cost], [\$18,000], [\$40,000],
    [Rover lifespan], [5 years], [2.5 years],
    [Fleet uptime], [90%], [70%],
  ),
  caption: [Sensitivity analysis: break-even points vs contractor baseline],
) <tab:sensitivity>

The system remains cost-competitive even under pessimistic assumptions. At 1:1 teleoperation (current capability), robotic systems still beat contractors by 22% due to eliminated markup and consistent productivity.

== Summary

#figure(
  table(
    columns: 3,
    align: (left, right, right),
    table.header([*Metric*], [*vs Manual*], [*vs Contractor*]),
    [5-year TCO reduction (1:10)], [69%], [49%],
    [5-year TCO reduction (1:1)], [53%], [22%],
    [Payback period (1:10)], [11 months], [16 months],
    [Payback period (1:1)], [14 months], [21 months],
  ),
  caption: [Economic summary (50 miles, 20 events/season)],
) <tab:econ-summary>

Robotic sidewalk maintenance is economically viable at *both* current (1:1) and target (1:10) autonomy levels. The difference is magnitude: supervised autonomy doubles the savings. Additional value from liability reduction and injury avoidance is not included in these figures but can be substantial.

= Governance, Data, and Vendor Risk

This section addresses questions that arise in procurement: Who owns what? What happens if the vendor fails? How do we exit?

== Data Ownership

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Data Type*], [*Owner*], [*Retention*]),
    [Telemetry], [Customer], [90 days standard],
    [Video recordings], [Customer], [30 days standard],
    [Route and map data], [Customer], [Indefinite],
    [Fleet analytics], [Vendor (anonymized)], [Indefinite],
    [Firmware/software], [Vendor (licensed)], [Escrow available],
  ),
  caption: [Data ownership and retention],
) <tab:data>

Customers can export all operational data at any time in standard formats (CSV, JSON, GeoJSON).

== Auditability

The system is designed for public accountability: open logs (clearing routes, times, and coverage are exportable), incident reports (full documentation for any flagged event), performance metrics (uptime, coverage completion, response times), and third-party audit availability on request.

== Vendor Continuity Risk

*Hardware:* Rovers are owned by the customer. Hardware is based on commodity components. Third-party maintenance is feasible.

*Software:* Firmware source code is held in escrow. In the event of vendor dissolution, escrow is released to customers with active support contracts.

*Transition period:* Vendor commits to 12 months notice before discontinuing support for any product generation.

== Exit Strategy

Customers can exit the system at any time. Hardware can be resold, repurposed, or disposed. All data is exportable in standard formats. Annual software subscriptions can be cancelled with 30 days notice.

= Roadmap

This section describes what capabilities are expected to improve over time, without specifying timelines that cannot be guaranteed.

== What Improves with Software

Autonomy level is the primary software-gated capability. Current systems require 1:1 teleoperation. As perception and planning algorithms mature, the operator-to-rover ratio will increase to 1:2, then 1:5, then 1:10. Each transition requires demonstrated reliability over a full season before deployment. Route optimization, fleet coordination, and predictive maintenance also improve with software updates and accumulated operational data.

== What Requires Hardware Revision

Clearing width, battery capacity, and sensor range are hardware-constrained. The current platform clears a 24-inch path. Wider clearing requires a new chassis generation. Battery capacity improvements depend on cell technology advances and are expected at 5–10% per year. Sensor upgrades (higher-resolution LiDAR, thermal cameras) require hardware swaps but are designed to be field-installable.

== What Is Constrained by Physics

Snow clearing rate is fundamentally limited by auger capacity and forward speed. The current platform clears approximately 0.5 miles per hour in 4-inch snow. Doubling this rate would require either a wider auger (which exceeds sidewalk width constraints) or faster forward speed (which reduces clearing quality and increases pedestrian risk). Battery energy density limits range. Current lithium-ion technology provides approximately 4 hours of continuous operation in cold weather. Step-change improvements require new battery chemistry.

== What Depends on Regulation

Autonomous operation in public rights-of-way is subject to state and local regulation. As of December 2024, 14 states have personal delivery device legislation. Expansion to other jurisdictions requires either legislative action or municipal pilot agreements. The system is designed to comply with the most restrictive current requirements, ensuring broad deployability as regulations evolve.

= The Path to Full Autonomy

This section addresses a question that sophisticated readers will ask: where does this end? The answer is full autonomy, rovers that clear sidewalks without human oversight. This section explains why we believe this is achievable, what technical and regulatory gates must be passed, and why we are building toward it even though we are not there today.

== Why Full Autonomy Matters

The economics of robotic sidewalk maintenance scale with the operator-to-rover ratio. At 1:1 (current), the system provides coverage consistency and reduced injury risk but does not reduce labor cost. At 1:10 (near-term target), labor cost drops by 90%. At 1:50 or higher (full autonomy), marginal labor cost approaches zero.

At full autonomy, the cost structure inverts. Sidewalk clearing becomes a capital and energy problem rather than a labor problem. A municipality could clear 200 miles of sidewalk with a fleet of 40 rovers, zero operators during clearing, and one maintenance technician. Seasonal labor shortages become irrelevant. Response time becomes a function of fleet size and charging infrastructure, not staff availability.

This is not a marginal improvement. It is a category change in how sidewalk maintenance can be delivered.

== Technical Requirements

Full autonomy requires capabilities beyond current state-of-the-art:

*Perception in degraded conditions.* Snow, fog, darkness, and glare all reduce sensor effectiveness. Current LiDAR and camera systems work well in moderate conditions but degrade in heavy precipitation. Full autonomy requires sensor fusion and learned perception that maintain safe operation across the full environmental envelope.

*Edge case handling.* Supervised autonomy allows operators to intervene for unusual situations: a car parked on the sidewalk, construction barriers, a fallen tree. Full autonomy requires the rover to recognize these situations, plan around them, or safely abort and retry. The long tail of edge cases is the primary technical challenge.

*Night and low-visibility operation.* Snow events often occur at night. Clearing before morning commute requires operation in darkness. This is achievable with current sensors but requires additional validation.

*Multi-rover coordination.* At scale, rovers must avoid interfering with each other, hand off routes efficiently, and coordinate around shared obstacles. This is a solved problem in warehouse robotics but less tested in outdoor environments.

*Graceful degradation.* When the system cannot proceed safely, it must fail in a way that does not create new hazards. A rover stopped in the middle of a sidewalk is a problem. Full autonomy requires planning for failure states as carefully as success states.

== The Liability Shift

Under supervised autonomy, operator error is a plausible cause for any incident. The operator saw (or should have seen) the pedestrian. Under full autonomy, this defense disappears. Every incident becomes a potential product liability claim.

This is not a reason to avoid full autonomy. It is a reason to reach it only through demonstrated safety. The path is:

+ Accumulate millions of operational hours under supervision
+ Document incident rates, near-misses, and intervention frequency
+ Demonstrate that autonomous operation is *safer* than supervised operation (fewer interventions, faster stops, more consistent behavior)
+ Obtain regulatory approval based on this evidence

The precedent is aviation autopilot: full autonomy was achieved not by claiming safety in advance, but by demonstrating it over decades of incremental deployment.

== Regulatory Path

No jurisdiction currently permits unsupervised robotic operation on public sidewalks. Personal delivery device (PDD) legislation typically requires a human operator capable of monitoring and taking control. This is appropriate given current technology.

The regulatory path forward has two components:

*Demonstrated safety record.* Regulators respond to evidence. Years of supervised operation with low incident rates create the foundation for expanded permissions.

*Pilot-to-permanent frameworks.* Several states have enacted pilot programs that allow expanded autonomy under controlled conditions. These provide a testing ground for full autonomy without requiring legislative change.

We expect full autonomy to be permitted in some jurisdictions within 5–7 years, following the pattern of autonomous vehicle regulation: early pilots in permissive jurisdictions, gradual expansion based on safety data, eventual standardization.

== Why We Build for It Now

The system described in this paper is designed for full autonomy even though it operates today under human supervision. This is intentional.

*Sensor and compute overhead.* The rover carries more perception capability than 1:1 teleoperation requires. This overhead enables autonomy development without hardware revision.

*Data collection.* Every supervised operation generates training data for autonomous systems. Routes, obstacles, interventions, and edge cases are logged and available for model development.

*Fail-safe architecture.* The safety systems (watchdog, E-stop, obstacle detection) are designed assuming no human is watching. This is the correct assumption for full autonomy and a conservative assumption for supervised operation.

*Fleet coordination layer.* The base station already manages multi-rover dispatch, route assignment, and status monitoring. These systems scale to full autonomy without architectural change.

The result is a system that can transition from supervised to autonomous operation through software updates, not hardware redesign. This is the strategic foundation for long-term cost advantage.

== Timeline Honesty

We do not provide a timeline for full autonomy. Too many variables are outside our control: regulatory frameworks, sensor technology, insurance markets, and public acceptance.

What we can say:

- 1:2 assisted teleoperation is achievable within 12 months with current technology
- 1:10 supervised autonomy requires 18–24 months of development and validation
- Full autonomy (1:50+) is a multi-year effort dependent on regulatory progress

We are building a company, not a demo. The path to full autonomy is measured in years and validated in operational hours, not press releases.

= Conclusion

Municipal sidewalk maintenance is a constrained optimization problem. Labor is scarce, seasonal, and expensive. Equipment designed for roadways cannot operate on sidewalks. The result is a persistent service gap.

Robotic systems can close this gap when the system operates reliably in the target environment (verified through pilot), supervised autonomy is achieved (one operator monitoring multiple units), total cost of ownership is lower than alternatives, and safety and liability frameworks are acceptable to the deploying organization. The system described in this paper is designed to meet these conditions. It is operational today in pilot configuration. Specifications in this document reflect current capabilities, not roadmap projections.

Municipal robotics is viable only if treated as infrastructure: reliable, maintainable, accountable, and boring. This system is designed accordingly.

#v(1em)
#align(center)[
  #rect(
    width: 80%,
    inset: 1em,
    stroke: 1pt + gray,
    radius: 4pt,
  )[
    *Pilot Program Inquiries* \
    #v(0.3em)
    Muni is accepting pilot partners for the 2026–2027 winter season. \
    Municipalities with 50+ miles of sidewalk and interest in operational evaluation are invited to inquire. \
    #v(0.3em)
    #link("mailto:info@muni.works")[info\@muni.works] · #link("https://muni.works")[muni.works]
  ]
]

#pagebreak()

= Appendix: Case Study, Lakewood, Ohio

This appendix applies the economic model to a specific municipality using publicly available data.

== City Profile

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Parameter*], [*Value*], [*Source*]),
    [Population], [49,517], [Census (2024)],
    [Area], [5.5 sq mi], [Census],
    [Population density], [\~9,000/sq mi], [Highest in Ohio],
    [Sidewalk network], [180+ miles], [City of Lakewood],
    [Street network], [90 miles], [City of Lakewood],
    [Snow events (1"+)], [24/season], [NOAA],
  ),
  caption: [Lakewood, Ohio city profile],
) <tab:lakewood>

Lakewood is a first-ring suburb of Cleveland, located on Lake Erie. It is the most densely populated city in Ohio and has been recognized as the state's most walkable city. The city does not provide school busing; students walk to school, making sidewalk accessibility a public safety issue.

== Current Approach

*Legal framework:* Lakewood Codified Ordinance 521.06 requires property owners to clear sidewalks within 24 hours after snowfall ends.

*Enforcement:* Division of Housing and Building handles complaints.

*Assistance:* LakewoodAlive operates a volunteer snow removal program for seniors and residents with disabilities.

*Municipal clearing:* None. The city clears streets but not sidewalks.

== Priority Network Analysis

Rather than attempting to clear all 180 miles, a robotic system would focus on a priority network:

#figure(
  table(
    columns: 3,
    align: (left, right, left),
    table.header([*Route Category*], [*Miles*], [*Rationale*]),
    [School walking routes], [25], [Student safety],
    [Commercial districts], [12], [Economic activity],
    [Transit corridors], [8], [Accessibility],
    [Senior/disabled housing], [5], [Equity, ADA],
    [*Total*], [*50*], [],
  ),
  caption: [Priority network for Lakewood],
) <tab:priority>

This represents 28% of the total network but covers the highest-liability and highest-visibility segments.

== Cost Comparison

Lakewood's 24-event season (vs 20 in the base model) increases both manual costs and robotic operating hours proportionally. Fleet sizing: 15 rovers (50 mi ÷ 4 mi/rover + N+2 redundancy).

#figure(
  table(
    columns: 4,
    align: (left, right, right, right),
    table.header([*Approach*], [*Year 1*], [*Years 2–5*], [*5-Year TCO*]),
    [Manual (hypothetical)], [\$400,000], [\$400,000/yr], [\$2,000,000],
    [Contractor (\$200/mi)], [\$240,000], [\$240,000/yr], [\$1,200,000],
    [Robotic (1:1 teleop)], [\$385,000], [\$115,000/yr], [\$845,000],
    [Robotic (1:10 supervised)], [\$319,000], [\$49,000/yr], [\$515,000],
  ),
  caption: [Lakewood 5-year TCO comparison (50-mile priority network, 24 events/season)],
) <tab:lakewood-tco>

#figure(
  lq.diagram(
    width: 6cm,
    height: 3cm,
    xaxis: (ticks: ((0, [Manual]), (1, [Contract]), (2, [1:1]), (3, [1:10]))),
    yaxis: (label: [TCO (\$M)]),
    lq.bar(
      (0, 1, 2, 3),
      (2.00, 1.20, 0.85, 0.52),
      width: 0.6,
      fill: rgb("#4caf50"),
    ),
  ),
  caption: [Lakewood 5-year TCO: supervised autonomy is 57% cheaper than contractors],
) <fig:lakewood-tco>

At supervised autonomy (1:10), robotic systems reduce 5-year TCO by *74% vs manual labor* and *57% vs contractors*. Even at 1:1 teleoperation, the system beats contractors by 30%. The higher savings compared to the base model reflect Lakewood's above-average snow frequency.

== Recommended Pilot

*Scope:* 8 miles of school walking routes (2 rovers × 4 mi/event)

*Duration:* One full winter season (December–March)

*Fleet:* 3 rovers (2 active, 1 spare)

*Capital cost:* 3 × \$18,000 = \$54,000

*Operating cost (1:1 teleop):* 3 × \$6,785 = \$20,400 for the season

*Total pilot investment:* ~\$75,000

*Evaluation criteria:* Coverage completion rate (target: 95%+), clearing time per event (target: 8 hours), uptime during events (target: 85%+), incident rate (target: zero), resident feedback.

*Decision point:* If pilot succeeds, expand to full priority network (50 miles, 15 rovers) in Year 2 with demonstrated path to 1:10 supervision.

#pagebreak()

= Appendix: Environmental and Operational Specifications

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Parameter*], [*Specification*], [*Notes*]),
    [Operating temperature], [−20°F to 40°F (−29°C to 4°C)], [Battery capacity reduced \~30% at low end],
    [Storage temperature], [−40°F to 120°F], [Requires climate-controlled charging],
    [Precipitation], [IP65 rated], [Continuous operation in snow, rain, sleet],
    [Snow depth (clearing)], [Up to 6 inches per pass], [Deeper accumulations require multiple passes],
    [Snow depth (navigation)], [Up to 12 inches], [Beyond this, navigation sensors obscured],
    [Grade/slope], [Up to 8% (1:12)], [ADA-compliant ramps; steeper requires speed reduction],
    [Surface types], [Concrete, asphalt, pavers], [Gravel and grass not supported],
    [Sidewalk width], [Minimum 36 inches], [ADA minimum; narrower requires manual clearing],
  ),
  caption: [Environmental operating envelope],
) <tab:env-specs>

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Parameter*], [*Specification*], [*Notes*]),
    [Continuous operation], [4 hours at 20°F], [Reduced to 2.5 hours at −20°F],
    [Charge time], [2 hours (0–80%)], [Full charge 3 hours],
    [Clearing rate], [0.5 mi/hr (4" snow)], [0.3 mi/hr in 6" snow],
    [Maximum speed], [1.2 m/s (2.7 mph)], [Reduced near pedestrians],
    [Obstacle detection range], [10 m (LiDAR)], [Reduced in heavy precipitation],
    [Communication range (WiFi)], [500 m line-of-sight], [Extended with mesh repeaters],
    [Communication range (LTE)], [Carrier-dependent], [Requires cellular coverage],
    [Data logging], [90 days telemetry], [30 days video; events indefinite],
  ),
  caption: [Operational specifications],
) <tab:op-specs>

#figure(
  table(
    columns: 2,
    align: (left, left),
    table.header([*Component*], [*Expected Lifetime*]),
    [Chassis/frame], [10+ years],
    [Drivetrain (motors, gearboxes)], [5 years / 2,000 hours],
    [Battery pack], [3 years / 1,000 cycles],
    [Electronics (compute, CAN)], [5 years],
    [Sensors (LiDAR, cameras)], [5 years],
    [Auger attachment], [3 seasons / 500 hours],
    [Tires], [2 seasons],
  ),
  caption: [Component lifetime estimates],
) <tab:lifetime>

#pagebreak()
#heading(outlined: true, numbering: none)[References]
#bibliography("refs.bib", style: "ieee")
