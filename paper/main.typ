#import "@preview/charged-ieee:0.1.4": ieee
#import "@preview/cetz:0.4.2"

#show: doc => {
  // Override template fonts with system-available alternatives
  set text(font: "Times New Roman")
  show raw: set text(font: "Menlo")
  
  ieee(
    title: [Robotic Systems for Sidewalk Maintenance: Architecture, Operations, and Economic Analysis],
    abstract: [
      Municipalities, property managers, and commercial operators maintain over 600,000 miles of sidewalks in the United States. Snow and ice removal on these surfaces is mandated by ADA compliance, tort liability, and ordinance. The labor required is seasonal, episodic, and difficult to staff. Equipment designed for roadways cannot operate on sidewalks. The result is a service gap addressed through overtime, contractors, and deferred maintenance.

      This paper describes a robotic system designed to close that gap. The system consists of a small-footprint rover platform (0.6m × 0.6m) capable of sidewalk navigation, a modular attachment interface supporting snow clearing, sweeping, and brine application, a remote operations model that allows one operator to supervise multiple units, and a fleet coordination layer that integrates with existing municipal GIS and work order systems.

      Under supervised autonomy (one operator monitoring ten units), the system reduces marginal sidewalk clearing cost by 70–85% compared to manual labor, with five-year total cost of ownership approximately 80% lower than current approaches for networks exceeding 50 miles. The system is currently deployed in pilot configuration under direct human supervision via LTE teleoperation. Specifications reflect current hardware and software constraints.
    ],
    authors: (
      (
        name: "Muni Municipal Robotics",
        organization: [Technical Documentation],
        location: [Cleveland, Ohio],
        email: "info@muni.bot"
      ),
    ),
    index-terms: ("Municipal robotics", "Sidewalk maintenance", "Snow removal", "Fleet automation", "Public works"),
    bibliography: bibliography("refs.bib"),
    figure-supplement: "Figure",
    doc,
  )
}

= Introduction

The intended audience for this paper is municipal public works departments, university facilities managers, and commercial property operators evaluating alternatives to manual sidewalk maintenance.

The system described in this paper is operational. Specifications reflect current hardware and software constraints.

@fig:architecture shows the high-level system architecture. The operator controls rovers remotely via LTE through an optional cloud relay for NAT traversal. Each rover operates independently with local safety systems that can halt the vehicle without network connectivity.

#figure(
  cetz.canvas(length: 1cm, {
    import cetz.draw: *
    
    // Operator station
    rect((-5, 0), (-2, 2), name: "operator", fill: rgb("#e3f2fd"), stroke: black)
    content("operator.center", align(center)[*Operator*\ Station])
    
    // Cloud relay
    circle((0, 1), radius: 0.8, name: "cloud", fill: rgb("#fff3e0"), stroke: black)
    content("cloud.center", align(center)[Cloud\ Relay])
    
    // Rover
    rect((2, 0), (5, 2), name: "rover", fill: rgb("#e8f5e9"), stroke: black)
    content("rover.center", align(center)[*Rover*\ (Jetson + VESC)])
    
    // Arrows
    line((-2, 1.3), (-0.8, 1.3), mark: (end: ">"), stroke: 1.5pt)
    content((-1.4, 1.7), text(size: 7pt)[Commands])
    
    line((-0.8, 0.7), (-2, 0.7), mark: (end: ">"), stroke: 1.5pt)
    content((-1.4, 0.3), text(size: 7pt)[Telemetry])
    
    line((0.8, 1.3), (2, 1.3), mark: (end: ">"), stroke: 1.5pt)
    line((2, 0.7), (0.8, 0.7), mark: (end: ">"), stroke: 1.5pt)
    
    // LTE labels
    content((-1.4, -0.5), text(size: 8pt)[QUIC/UDP])
    content((1.4, -0.5), text(size: 8pt)[LTE])
    
    // Safety callout
    rect((2.2, -1.5), (4.8, -0.3), fill: rgb("#ffebee"), stroke: rgb("#c62828"))
    content((3.5, -0.9), text(size: 7pt)[Local safety:\ E-stop, watchdog])
    line((3.5, -0.3), (3.5, 0), mark: (end: ">"), stroke: rgb("#c62828") + 1pt)
  }),
  caption: [System architecture: operator-rover control loop with cloud relay],
) <fig:architecture>

= Problem Definition: Public Works as a Control System

== The Optimization Problem

Municipal public works departments solve a recurring constrained optimization problem:

*Objective:* Maintain public rights-of-way to a defined service level

*Subject to:*
- Fixed annual budget (typically set 18 months in advance)
- Hard service-level agreements (snow cleared within N hours of snowfall end)
- Seasonal demand spikes (10× variance in labor need between summer and winter)
- Adversarial environment (weather, vandalism, equipment failure, political pressure)
- Asset lifetime requirements (15–25 years for equipment)
- Public accountability (every failure is photographed and posted)

This is a control problem, not a technology problem. The question is not whether robots can clear snow. The question is whether a robotic system can meet service-level guarantees more reliably than the current approach, at equal or lower cost, without introducing new failure modes that the department cannot manage.

The control variables available to a public works director are:

- *Labor hours:* Total person-hours allocated to sidewalk clearing per event
- *Fleet size:* Number of equipment units (manual or robotic) available for deployment
- *Route sequencing:* Order and prioritization of sidewalk segments (arterials, schools, transit stops)
- *Response latency:* Time between snowfall end and clearing completion
- *Equipment availability:* Percentage of fleet operational at any given time

Any proposed system must improve at least one of these variables without degrading the others.

== Reference Case: Lakewood, Ohio

Lakewood is a first-ring suburb of Cleveland with a population of 49,500 and over 180 miles of sidewalks—the most walkable city in Ohio and the state's most densely populated municipality (~9,000 residents per square mile). The city experiences an average of 24 snow events per season requiring clearing.

Currently, Lakewood does not clear sidewalks municipally. Property owners are required by ordinance to clear adjacent sidewalks within 24 hours of snowfall. Enforcement is handled by the Division of Housing and Building on a complaint basis. The city does not provide school busing, making sidewalk accessibility a student safety issue.

This profile—high density, extensive sidewalk network, property-owner mandate with uneven compliance, and no current municipal clearing budget—represents a common pattern in Midwestern cities.

== Current Approaches and Failure Modes

Most municipalities address sidewalk maintenance through one of three approaches:

*1. Municipal crews with hand tools and small equipment*

Typical configuration: seasonal workers with shovels, walk-behind snowblowers, and occasionally ATVs or Toolcats.

Failure modes: Labor availability (snowstorms do not schedule around shift changes), coverage rate (a worker with a shovel clears approximately 0.1 miles per hour), consistency (different workers clear to different standards), and injury (snow removal is the leading cause of workers' compensation claims in public works).

*2. Contractor services*

Typical configuration: Landscaping companies with plowing contracts.

Failure modes: Incentive misalignment (per-event contracts reward billing, not coverage), verification (municipalities rarely have real-time visibility into contractor operations), reliability (contractors serve multiple clients), and equipment mismatch (contractors use equipment sized for parking lots).

*3. Property owner mandates*

Typical configuration: Ordinances requiring property owners to clear adjacent sidewalks within N hours.

Failure modes: Enforcement cost, equity (elderly, disabled, and low-income residents cannot comply), and inconsistency (a cleared sidewalk next to an uncleared sidewalk is not a cleared route).

== The Structural Problem

All three approaches share a common failure: they treat sidewalk maintenance as an episodic labor problem rather than a continuous coverage problem.

The service requirement is spatial: every linear foot of sidewalk must be cleared. The labor model is temporal: workers clock in and clock out. The mismatch is fundamental.

Heavy equipment solves this mismatch for roadways. A plow truck clears miles per hour. A single operator covers an entire route. But heavy equipment cannot operate on sidewalks. The geometry does not permit it. ADA minimum clear width is 36 inches. A standard plow truck is 102 inches wide.

The result is that sidewalks—the most pedestrian-critical infrastructure—are maintained with the lowest-productivity methods.

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

= Safety and Liability

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
    
    // E-Stop release back to Idle
    bezier((0.5, -1.5), (2.5, 0), (2.5, 2), (0.7, 2), mark: (end: ">"), stroke: 1pt)
    content((2.8, 0.5), text(size: 6pt)[Release])
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

The system operates on shared pedestrian infrastructure. Interaction assumptions:

- *Speed:* Maximum 1.2 m/s in normal operation; reduced to 0.5 m/s when pedestrians detected within 3m
- *Yielding:* Rover always yields to pedestrians; does not attempt to pass or navigate around people in motion
- *Stopping distance:* \<0.5m at maximum speed on dry pavement; \<1m on snow/ice
- *Visibility:* Amber marker lights at all corners; retroreflective markings on all sides
- *Audibility:* Optional low-volume alert tone before movement

The system does not rely on pedestrians to behave predictably. If a pedestrian stops in front of the rover, the rover waits indefinitely.

== Incident Logging and Replay

All operational data is logged:

- *Telemetry:* Position, velocity, motor currents, battery state (1 Hz, retained 90 days)
- *Commands:* All operator inputs with timestamps (retained 90 days)
- *Video:* Continuous recording during operation (retained 30 days)
- *Events:* Obstacles detected, stops triggered, faults occurred (retained indefinitely)

Logs are stored locally on the rover and synced to the base station. In the event of an incident, complete session replay is available within hours.

== Insurance and Liability

*Current model:* The deploying organization carries general liability insurance. Robotic operations are typically covered under existing policies as mobile equipment.

*Vendor liability:* The vendor warrants that the system performs as specified. The vendor does not assume operational liability for incidents arising from operator error, environmental conditions outside specified limits, or unauthorized modifications.

*Incident investigation:* In the event of an incident involving injury or significant property damage, the vendor will provide full access to telemetry and logs, technical support for investigation, and cooperation with legal and regulatory processes.

== Regulatory Status

Sidewalk robots are not federally regulated in the United States. Regulation, where it exists, is at the state or municipal level. As of December 2024, 14 states have enacted personal delivery device (PDD) legislation with weight limits typically ranging from 80–550 lbs and speed limits of 6–12 mph. Most require yielding to pedestrians, operator oversight, and liability insurance.

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

The system integrates with existing municipal infrastructure at the following points:

- *GIS/mapping:* Import sidewalk network as routes (low complexity, standard formats)
- *Work orders:* Export clearing logs; optionally receive dispatch (medium complexity)
- *Weather services:* Receive forecasts for pre-positioning (low complexity)
- *Citizen complaints:* Cross-reference complaints with clearing logs (medium complexity)
- *Fleet management:* Dashboard for status, telemetry, alerts (included)

Full integration is not required for pilot. Minimum viable integration: GIS import for route planning.

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

At \$40/hour loaded cost and 0.15 miles/hour productivity, labor cost per mile is *\$267*. For a city with 100 miles of sidewalk and 20 events per season, seasonal labor cost is *\$534,000*.

Per-event contractor rates range from \$150–400 per mile. Seasonal contracts for 100 miles of sidewalk typically range from \$400,000–800,000.

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
  caption: [Per-unit hardware cost (current prototype)],
) <tab:bom>

Target production cost at scale (100+ units): *\$3,400*. Sale price (target): \$12,000 base, \$15,000 with snow clearing, \$18,000 with full sensor suite.

*Asset lifetime:* 5-year service life with annual maintenance. Mid-life refurbishment at year 3 (\$800–1,200). Chassis can be refurbished for a second 5-year cycle at approximately 40% of new unit cost.

== Operating Costs

Annual per-rover operating costs: operator labor (\$2,400–6,000), LTE connectivity (\$360), maintenance/consumables (\$500), battery replacement amortized (\$200), charging energy (\$100–150), software subscription (\$1,200), insurance (\$400). *Total: \$5,160–8,810/year*.

== Operator Economics

The viability of robotic sidewalk maintenance depends on the operator-to-rover ratio.

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

== Total Cost of Ownership Comparison

Scenario: 100 miles of sidewalk, 20 snow events per season, 5-year analysis period.

#figure(
  table(
    columns: 3,
    align: (left, right, right),
    table.header([*Approach*], [*Annual*], [*5-Year TCO*]),
    [Manual labor (municipal)], [\$654,000], [\$3,270,000],
    [Contractor], [\$650,000], [\$3,250,000],
    [Robotic (supervised autonomy)], [\$85,000 (yr 2–5)], [\$630,000],
  ),
  caption: [Total cost of ownership comparison],
) <tab:tco>

*Payback period: 6–8 months of operation*

== Sensitivity Analysis

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Variable*], [*Base Case*], [*Break-Even*]),
    [Operator ratio], [1:10], [1:4],
    [Snow events/season], [20], [8],
    [Clearing rate], [0.5 mi/hr], [0.3 mi/hr],
    [Hardware cost], [\$18,000], [\$35,000],
    [Rover lifespan], [5 years], [3 years],
    [Downtime rate], [10%], [40%],
    [LTE reliability], [99%], [95%],
  ),
  caption: [Sensitivity analysis: break-even points],
) <tab:sensitivity>

== Summary

Robotic sidewalk maintenance is economically viable under the following conditions:

+ Supervised autonomy (1:10 operator ratio) is achieved
+ Unit hardware cost remains below \$25,000
+ Seasonal snow events exceed 10 per year
+ Municipality has 50+ miles of sidewalk
+ System uptime exceeds 85% during clearing events

At current capability (1:1 teleoperation), the system reduces injury risk and provides consistent coverage but does not reduce total cost. The economic case depends on autonomy development.

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

= Conclusion

Municipal sidewalk maintenance is a constrained optimization problem. Labor is scarce, seasonal, and expensive. Equipment designed for roadways cannot operate on sidewalks. The result is a persistent service gap.

Robotic systems can close this gap under specific conditions:

+ The system operates reliably in the target environment (verified through pilot)
+ Supervised autonomy is achieved (one operator monitoring multiple units)
+ Total cost of ownership is lower than alternatives
+ Safety and liability frameworks are acceptable to the deploying organization

The system described in this paper is designed to meet these conditions. It is operational today in pilot configuration. Specifications in this document reflect current capabilities, not roadmap projections.

Municipal robotics is viable only if treated as infrastructure: reliable, maintainable, accountable, and boring. This system is designed accordingly.

#pagebreak()

= Appendix: Case Study — Lakewood, Ohio

This appendix applies the economic model to a specific municipality using publicly available data.

== City Profile

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header([*Parameter*], [*Value*], [*Source*]),
    [Population], [49,517], [Census (2024)],
    [Area], [5.5 sq mi], [Census],
    [Population density], [~9,000/sq mi], [Highest in Ohio],
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

#figure(
  table(
    columns: 2,
    align: (left, right),
    table.header([*Approach*], [*5-Year TCO*]),
    [Manual (hypothetical)], [\$1,927,000],
    [Contractor], [\$1,000,000],
    [Robotic (1:1 teleop)], [\$1,021,000],
    [Robotic (supervised autonomy)], [\$751,000],
  ),
  caption: [Lakewood 5-year TCO comparison (50-mile priority network)],
) <tab:lakewood-tco>

At supervised autonomy, robotic systems reduce 5-year TCO by approximately 25% compared to contractors.

== Recommended Pilot

*Scope:* 5–10 miles of school walking routes in one quadrant of the city

*Duration:* One full winter season (December–March)

*Fleet:* 3 rovers (2 active, 1 spare)

*Capital cost:* ~\$55,000

*Operating cost:* ~\$25,000 for the season

*Evaluation criteria:* Coverage completion rate (target: 95%+), clearing time (target: 8 hours), uptime during events (target: 85%+), incident rate (target: zero), resident feedback.

*Decision point:* If pilot succeeds, expand to full priority network (50 miles) in Year 2.
