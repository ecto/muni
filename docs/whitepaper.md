# Robotic Systems for Sidewalk Maintenance: Architecture, Operations, and Economic Analysis

---

## 1. Abstract

Municipalities, property managers, and commercial operators maintain over 600,000 miles of sidewalks in the United States. Snow and ice removal on these surfaces is mandated by ADA compliance, tort liability, and ordinance. The labor required is seasonal, episodic, and difficult to staff. Equipment designed for roadways cannot operate on sidewalks. The result is a service gap addressed through overtime, contractors, and deferred maintenance.

This paper describes a robotic system designed to close that gap. The intended audience is municipal public works departments, university facilities managers, and commercial property operators evaluating alternatives to manual sidewalk maintenance.

The system consists of:

- A small-footprint rover platform (0.6m × 0.6m) capable of sidewalk navigation
- A modular attachment interface supporting snow clearing, sweeping, and brine application
- A remote operations model that allows one operator to supervise multiple units
- A fleet coordination layer that integrates with existing municipal GIS and work order systems

Under supervised autonomy (one operator monitoring ten units), the system reduces marginal sidewalk clearing cost by 70–85% compared to manual labor, with five-year total cost of ownership approximately 80% lower than current approaches for networks exceeding 50 miles.

The system is currently deployed in pilot configuration. It operates under direct human supervision via LTE teleoperation. Autonomous capabilities are limited to safety functions: obstacle detection, emergency stop, and operator-loss failsafe. The platform is designed to increase autonomy incrementally as reliability is demonstrated.

The system described in this paper is operational. Specifications reflect current hardware and software constraints.

---

## 2. Problem Definition: Public Works as a Control System

### 2.1 The Optimization Problem

Municipal public works departments solve a recurring constrained optimization problem:

**Objective:** Maintain public rights-of-way to a defined service level
**Subject to:**

- Fixed annual budget (typically set 18 months in advance)
- Hard service-level agreements (snow cleared within N hours of snowfall end)
- Seasonal demand spikes (10× variance in labor need between summer and winter)
- Adversarial environment (weather, vandalism, equipment failure, political pressure)
- Asset lifetime requirements (15–25 years for equipment)
- Public accountability (every failure is photographed and posted)

This is a control problem, not a technology problem. The question is not whether robots can clear snow. The question is whether a robotic system can meet service-level guarantees more reliably than the current approach, at equal or lower cost, without introducing new failure modes that the department cannot manage.

The control variables available to a public works director are:

- **Labor hours**: Total person-hours allocated to sidewalk clearing per event
- **Fleet size**: Number of equipment units (manual or robotic) available for deployment
- **Route sequencing**: Order and prioritization of sidewalk segments (arterials, schools, transit stops)
- **Response latency**: Time between snowfall end and clearing completion
- **Equipment availability**: Percentage of fleet operational at any given time

Any proposed system must improve at least one of these variables without degrading the others. The economic analysis in Section 9 quantifies these trade-offs.

### 2.2 Current Approach and Its Failure Modes

**Reference case: Lakewood, Ohio**

Lakewood is a first-ring suburb of Cleveland with a population of 49,500 and over 180 miles of sidewalks—the most walkable city in Ohio and the state's most densely populated municipality (~9,000 residents per square mile). The city experiences an average of 24 snow events per season requiring clearing.

Currently, Lakewood does not clear sidewalks municipally. Property owners are required by ordinance to clear adjacent sidewalks within 24 hours of snowfall. Enforcement is handled by the Division of Housing and Building on a complaint basis. The city does not provide school busing, making sidewalk accessibility a student safety issue.

This profile—high density, extensive sidewalk network, property-owner mandate with uneven compliance, and no current municipal clearing budget—represents a common pattern in Midwestern cities. See Appendix A for detailed economic analysis of this case.

Most municipalities address sidewalk maintenance through one of three approaches:

**1. Municipal crews with hand tools and small equipment**

Typical configuration: seasonal workers with shovels, walk-behind snowblowers, and occasionally ATVs or Toolcats.

Failure modes:

- Labor availability. Snowstorms do not schedule themselves around shift changes. Overtime costs escalate rapidly.
- Coverage rate. A worker with a shovel clears approximately 0.1 miles per hour. A city with 200 miles of sidewalk requires 2,000 labor-hours per clearing event.
- Consistency. Different workers clear to different standards. ADA compliance is uneven.
- Injury. Snow removal is the leading cause of workers' compensation claims in public works.

**2. Contractor services**

Typical configuration: Landscaping companies with plowing contracts. Per-event or seasonal fixed-fee arrangements.

Failure modes:

- Incentive misalignment. Per-event contracts reward billing, not coverage. Seasonal contracts reward minimal passes.
- Verification. Municipalities rarely have real-time visibility into contractor operations. Disputes are resolved via complaint volume, not data.
- Reliability. Contractors serve multiple clients. Municipal sidewalks are often lowest priority behind commercial parking lots.
- Equipment mismatch. Contractors use equipment sized for parking lots. Sidewalks are an afterthought.

**3. Property owner mandates**

Typical configuration: Ordinances requiring property owners to clear adjacent sidewalks within N hours.

Failure modes:

- Enforcement cost. Enforcing residential snow ordinances consumes code enforcement resources.
- Equity. Elderly, disabled, and low-income residents cannot comply. The ordinance creates liability, not compliance.
- Inconsistency. A cleared sidewalk next to an uncleared sidewalk is not a cleared route.

### 2.3 The Structural Problem

All three approaches share a common failure: they treat sidewalk maintenance as an episodic labor problem rather than a continuous coverage problem.

The service requirement is spatial: every linear foot of sidewalk must be cleared. The labor model is temporal: workers clock in and clock out. The mismatch is fundamental.

Heavy equipment solves this mismatch for roadways. A plow truck clears miles per hour. A single operator covers an entire route. But heavy equipment cannot operate on sidewalks. The geometry does not permit it. ADA minimum clear width is 36 inches. A standard plow truck is 102 inches wide.

The result is that sidewalks—the most pedestrian-critical infrastructure—are maintained with the lowest-productivity methods.

### 2.4 Requirements for a Solution

Any system that claims to address this problem must satisfy the following constraints:

| Requirement        | Threshold                           | Rationale                                  |
| ------------------ | ----------------------------------- | ------------------------------------------ |
| Width              | ≤ 30 inches (762mm)                 | Operate within ADA minimum clear width     |
| Clearing rate      | ≥ 0.5 miles/hour                    | 5× hand labor productivity                 |
| Duty cycle         | ≥ 4 hours continuous                | Complete a route without returning to base |
| All-weather        | −20°F to 40°F, active precipitation | Operate when service is required           |
| Remote operability | LTE or equivalent                   | Supervise from central location            |
| Maintenance        | Field-serviceable                   | Repair without factory return              |
| Acquisition cost   | < $30,000                           | Justify against labor savings              |

These are not aspirational targets. They are minimum thresholds for operational viability.

### 2.5 What This Paper Does Not Claim

This paper does not claim that robotic sidewalk maintenance is superior to human labor in all circumstances. It claims that robotic systems can extend the coverage capacity of a fixed labor budget, reduce marginal cost per mile at scale, and provide consistent service levels that are difficult to achieve with variable labor.

The system described here is not autonomous in the consumer sense of that word. It requires human operators. It reduces the operator-to-asset ratio, not the operator count to zero.

The system does not eliminate the need for manual crews during extreme weather events. Blizzards, ice storms, and accumulations exceeding the system's clearing capacity (approximately 6 inches per pass) require conventional equipment and personnel. Robotic systems augment baseline capacity; they do not replace surge capacity.

---

## 7. Safety and Liability

This section addresses safety engineering and liability allocation. It is written for risk officers and city attorneys, not engineers.

### 7.1 Safety Design Philosophy

The system is designed to fail safe, not fail smart. When uncertainty exceeds thresholds, the rover stops. The priority order is:

1. Do not harm people
2. Do not damage property
3. Do not damage the rover
4. Complete the task

This ordering is enforced in firmware. Task completion is always the lowest priority.

### 7.2 Failure Modes and Responses

| Condition                        | Detection                 | Response                           | Recovery                                                    |
| -------------------------------- | ------------------------- | ---------------------------------- | ----------------------------------------------------------- |
| Obstacle detected                | LiDAR, camera             | Stop, assess, route around or wait | Automatic if path clears; operator escalation if persistent |
| Communication loss               | Heartbeat timeout (250ms) | Coast to stop, hold position       | Automatic resume when communication restored                |
| Operator loss (tab closed, etc.) | Heartbeat timeout         | Immediate zero velocity            | Resume when operator returns                                |
| Low battery                      | Voltage monitor           | Reduce speed, return to base       | Charge cycle                                                |
| Critical battery                 | Voltage monitor           | Safe stop, disable motors          | Manual recovery or tow                                      |
| Hardware fault                   | Self-diagnostics          | Safe stop, alert operator          | Manual inspection required                                  |
| E-stop activated                 | Operator command          | Immediate stop, motors disabled    | Requires explicit release command                           |

### 7.3 Pedestrian Interaction

The system operates on shared pedestrian infrastructure. Interaction assumptions:

- **Speed**: Maximum 1.2 m/s (walking pace) in normal operation; reduced to 0.5 m/s when pedestrians detected within 3m
- **Yielding**: Rover always yields to pedestrians; does not attempt to pass or navigate around people in motion
- **Stopping distance**: <0.5m at maximum speed on dry pavement; <1m on snow/ice
- **Visibility**: Amber marker lights at all corners; retroreflective markings on all sides; work light when tool active
- **Audibility**: Optional low-volume alert tone before movement; spoken announcement before passing ("Passing on your left")

The system does not rely on pedestrians to behave predictably. If a pedestrian stops in front of the rover, the rover waits indefinitely.

### 7.4 Incident Logging and Replay

All operational data is logged:

- **Telemetry**: Position, velocity, motor currents, battery state (1 Hz, retained 90 days)
- **Commands**: All operator inputs with timestamps (retained 90 days)
- **Video**: Continuous recording during operation (retained 30 days, longer if incident flagged)
- **Events**: Obstacles detected, stops triggered, faults occurred (retained indefinitely)

Logs are stored locally on the rover and synced to the base station. In the event of an incident, complete session replay is available within hours.

### 7.5 Insurance and Liability

**Current model**: The deploying organization (municipality, property manager) carries general liability insurance. Robotic operations are typically covered under existing policies as mobile equipment. Operators should confirm coverage with their insurer before deployment.

**Vendor liability**: The vendor warrants that the system performs as specified. The vendor does not assume operational liability for incidents arising from operator error, environmental conditions outside specified limits, or unauthorized modifications.

**Incident investigation**: In the event of an incident involving injury or significant property damage, the vendor will provide full access to telemetry and logs, technical support for investigation, and cooperation with legal and regulatory processes.

### 7.6 Regulatory Status

Sidewalk robots are not federally regulated in the United States. Regulation, where it exists, is at the state or municipal level. As of December 2024:

- 14 states have enacted personal delivery device (PDD) legislation
- Weight limits typically range from 80–550 lbs
- Speed limits typically 6–12 mph
- Most require yielding to pedestrians, operator oversight, and liability insurance

The system described in this paper is designed to comply with the most restrictive common requirements. Specific deployment jurisdictions may require permits, registration, or operational constraints.

---

## 8. Deployment and Integration

This section describes how the system is deployed in practice. It is written for operations managers and IT staff.

### 8.1 Pilot Sizing

Recommended pilot configuration:

| Component               | Quantity        | Notes                                         |
| ----------------------- | --------------- | --------------------------------------------- |
| Rovers                  | 2–3             | Allows direct comparison, provides redundancy |
| Attachments             | 1 per rover     | Match to season (snow auger for winter pilot) |
| Operator workstation    | 1               | Can supervise all pilot units                 |
| Charging infrastructure | 1 bay per rover | Co-located with storage                       |

**Pilot duration**: Minimum one full season (3–4 months for snow) to observe performance across weather conditions.

**Pilot scope**: 5–15 miles of sidewalk, selected for:

- Mix of conditions (residential, commercial, arterial)
- Accessible staging location
- Representative of broader network

### 8.2 Integration Touchpoints

The system integrates with existing municipal infrastructure at the following points:

| System             | Integration                                       | Complexity                                 |
| ------------------ | ------------------------------------------------- | ------------------------------------------ |
| GIS/mapping        | Import sidewalk network as routes                 | Low (standard formats: Shapefile, GeoJSON) |
| Work orders        | Export clearing logs; optionally receive dispatch | Medium (API or file export)                |
| Weather services   | Receive forecasts for pre-positioning             | Low (standard APIs)                        |
| Citizen complaints | Cross-reference complaints with clearing logs     | Medium (manual or API)                     |
| Fleet management   | Dashboard for status, telemetry, alerts           | Included                                   |

Full integration is not required for pilot. Minimum viable integration: GIS import for route planning.

### 8.3 Training

| Role        | Training time | Content                                                         |
| ----------- | ------------- | --------------------------------------------------------------- |
| Operator    | 4–8 hours     | Teleoperation, monitoring, exception handling, safety protocols |
| Supervisor  | 2–4 hours     | Fleet dashboard, reporting, escalation procedures               |
| Maintenance | 8–16 hours    | Inspection, consumable replacement, common repairs, diagnostics |

Training is provided on-site during commissioning. Refresher training recommended annually.

### 8.4 Seasonal Operations

**Pre-season (fall)**:

- Remove rovers from storage
- Inspect and replace worn components
- Update firmware
- Verify LTE connectivity
- Test routes

**In-season (winter)**:

- Deploy per weather forecast
- Charge between events
- Monitor for wear, damage

**Post-season (spring)**:

- Clean and inspect
- Remove batteries for storage (if long off-season)
- Store in climate-controlled area if possible
- Review season performance data

**Off-season utilization (optional)**:

- Swap attachments for sweeping, leaf collection
- Inspection routes (sidewalk condition assessment)
- Reduces effective capital cost per use-hour

### 8.5 Storage and Maintenance Facility

Requirements:

| Requirement | Specification                                                |
| ----------- | ------------------------------------------------------------ |
| Floor space | 100 sq ft per rover (includes charging, access)              |
| Power       | 20A 120V circuit per 2 rovers                                |
| Climate     | Above freezing preferred; required for battery storage       |
| Security    | Locked facility; rovers have GPS tracking and remote disable |
| Network     | Internet access for telemetry sync                           |

Most municipalities can accommodate pilots in existing public works facilities.

---

## 9. Economics

This section presents the economic case for robotic sidewalk maintenance. All figures are based on current hardware costs, observed productivity rates, and published municipal labor data. Sensitivity analysis follows.

### 9.1 Baseline: Current Municipal Costs

**Labor costs for manual sidewalk snow removal:**

| Parameter                                | Value                              | Source                               |
| ---------------------------------------- | ---------------------------------- | ------------------------------------ |
| Loaded labor rate (seasonal)             | $35–45/hour                        | BLS, municipal contracts             |
| Productivity (shovel)                    | 0.08–0.12 miles/hour               | Field observation (see note)         |
| Productivity (walk-behind blower)        | 0.15–0.25 miles/hour               | Manufacturer spec, field observation |
| Average snow events per season (Midwest) | 15–25                              | NOAA climate data                    |
| Clearing time requirement                | 4–12 hours post-snowfall (mode: 8) | Typical municipal ordinance          |

**Methodology note**: Productivity figures are based on timed clearing of marked sidewalk segments (100m intervals) across four snow events in Northeast Ohio during the 2024–2025 season. Measurements include transit time between segments but exclude breaks. Segment conditions ranged from 2–5 inches of dry snow. Figures represent median performance; variance is significant across workers and conditions.

**Cost per mile cleared (manual):**

At $40/hour loaded cost and 0.15 miles/hour productivity:

- Labor cost per mile: **$267**
- For a city with 100 miles of sidewalk and 20 events per season:
- Seasonal labor cost: **$534,000**

This excludes equipment, supervision, and workers' compensation.

**Contractor costs:**

Per-event contractor rates range from $150–400 per mile depending on region and snow depth. Seasonal contracts for 100 miles of sidewalk typically range from $400,000–800,000.

### 9.2 System Capital Costs

**Per-unit hardware cost (current prototype):**

| Category                        | Cost       |
| ------------------------------- | ---------- |
| Chassis and drivetrain          | $950       |
| Electronics (compute, CAN, LTE) | $890       |
| Perception (LiDAR, camera)      | $1,820     |
| Power system                    | $400       |
| Snow clearing attachment        | $365       |
| Assembly, wiring, integration   | $400       |
| **Total hardware cost**         | **$4,825** |

**Target production cost at scale (100+ units):**

| Category                          | Cost       |
| --------------------------------- | ---------- |
| Chassis and drivetrain            | $600       |
| Electronics                       | $700       |
| Perception                        | $1,200     |
| Power system                      | $350       |
| Snow clearing attachment          | $300       |
| Assembly (contract manufacturing) | $250       |
| **Total production cost**         | **$3,400** |

**Sale price (target):**

| Configuration              | Price   |
| -------------------------- | ------- |
| Base rover (no attachment) | $12,000 |
| Snow clearing package      | $15,000 |
| Full sensor suite + LiDAR  | $18,000 |

Gross margin at scale: 55–65%.

**Asset lifetime and refurbishment:**

The chassis and drivetrain are designed for a 5-year service life with annual maintenance. Key wear components (wheel bearings, motor brushes, battery pack) are field-replaceable. At year 3, a mid-life refurbishment ($800–1,200) extends service life by replacing worn drivetrain components. At year 5, the chassis can be refurbished for a second 5-year cycle at approximately 40% of new unit cost, or retired.

**Attachment classification:**

Attachments (snow auger, sweeper, brine sprayer) are capital assets with their own depreciation schedules. Auger blades and sweeper bristles are consumables, budgeted under operating costs. The attachment interface is standardized; attachments can be swapped seasonally or transferred between rovers.

### 9.3 Operating Costs

**Per-rover operating costs (annual):**

| Category                        | Cost             | Notes                                |
| ------------------------------- | ---------------- | ------------------------------------ |
| Operator labor                  | $2,400–6,000     | Depends on autonomy level (see 9.4)  |
| LTE connectivity                | $360             | Commercial IoT plan                  |
| Maintenance/consumables         | $500             | Tires, bearings, auger blades        |
| Battery replacement (amortized) | $200             | 3-year replacement cycle             |
| Charging infrastructure energy  | $100–150         | See note below                       |
| Software subscription           | $1,200           | Fleet management, updates            |
| Insurance                       | $400             | Estimated; will vary by jurisdiction |
| **Total annual operating cost** | **$5,160–8,810** |

**Charging energy note**: Each rover consumes approximately 1 kWh per operating hour. At 200 operating hours per season and $0.12/kWh commercial rate, annual energy cost is approximately $25. The $100–150 estimate includes charging equipment standby power, inefficiency losses, and a margin for heated storage during charging.

### 9.4 Operator Economics

The viability of robotic sidewalk maintenance depends on the operator-to-rover ratio. At 1:1, the system saves equipment costs but not labor costs. At 1:10, labor cost per rover-hour drops by 90%.

| Operating Mode         | Operator:Rover | Operator Cost/hr | Cost per Rover-Hour |
| ---------------------- | -------------- | ---------------- | ------------------- |
| Direct teleoperation   | 1:1            | $25              | $25.00              |
| Assisted teleoperation | 1:2            | $25              | $12.50              |
| Supervised autonomy    | 1:10           | $25              | $2.50               |
| Full autonomy          | 1:50+          | $25              | $0.50               |

**Current capability: Direct teleoperation (1:1)**

The system today operates at 1:1 ratio. Operator labor savings come from reduced physical labor (the operator is indoors, at a desk) and reduced injury risk, not from ratio improvement.

**Target capability: Supervised autonomy (1:10)**

At 1:10 ratio, a 10-rover fleet requires one operator during active clearing. For a 100-mile sidewalk network at 0.5 miles/hour clearing rate:

- Hours to clear: 200 rover-hours
- Operator hours: 20
- Labor cost: $500 (vs. $53,400 for manual labor)

This target requires completion of the following capability milestones:

1. Autonomous waypoint following along pre-mapped routes
2. Static obstacle detection and path replanning
3. Dynamic obstacle avoidance (pedestrians, vehicles)
4. Exception handling and operator escalation protocols

These capabilities are in active development. Deployment timing depends on validation requirements, which vary by jurisdiction.

**Labor considerations:**

Robotic systems change the nature of sidewalk maintenance labor; they do not eliminate it. Operators are typically drawn from existing public works staff and reassigned from physical clearing to supervisory roles. This transition offers several advantages: reduced physical strain, reduced injury exposure, and year-round employment stability (operators can supervise other equipment or perform other duties during off-season).

Departments with union representation should engage labor representatives early in pilot planning. The system is designed to augment crews, not replace headcount. In practice, the constraint is usually the opposite: departments cannot hire enough seasonal workers, and robotic systems fill the gap.

### 9.5 Cost Comparison: Total Cost of Ownership

**Scenario: 100 miles of sidewalk, 20 snow events per season, 5-year analysis period**

**Fleet sizing assumptions:**

- 10 rovers operating in parallel across geographically partitioned routes
- Each rover assigned 10 miles of sidewalk per event
- At 0.5 miles/hour, each rover clears its route in 20 hours
- Clearing occurs over two 10-hour shifts (overnight + morning) to meet 8-hour ordinance deadline for majority of network
- Routes are prioritized: arterials and transit stops first, residential segments second

**Base station cost breakdown ($30,000 initial):**

| Component                        | Cost    |
| -------------------------------- | ------- |
| Operator workstation (2 screens) | $3,000  |
| Fleet management software setup  | $5,000  |
| Charging infrastructure (10 bay) | $8,000  |
| Secure storage/maintenance area  | $10,000 |
| Network and LTE gateway          | $2,000  |
| Training and commissioning       | $2,000  |

| Approach                                     | Year 1       | Year 2–5        | 5-Year Total   |
| -------------------------------------------- | ------------ | --------------- | -------------- |
| **Manual labor (municipal)**                 |              |                 |                |
| Labor (20 events × 100mi × $267/mi)          | $534,000     | $534,000/yr     | $2,670,000     |
| Equipment (amortized)                        | $40,000      | $40,000/yr      | $200,000       |
| Workers' comp, overhead                      | $80,000      | $80,000/yr      | $400,000       |
| **Total**                                    | **$654,000** | **$654,000/yr** | **$3,270,000** |
|                                              |              |                 |                |
| **Contractor**                               |              |                 |                |
| Seasonal contract                            | $600,000     | $600,000/yr     | $3,000,000     |
| Oversight, complaints                        | $50,000      | $50,000/yr      | $250,000       |
| **Total**                                    | **$650,000** | **$650,000/yr** | **$3,250,000** |
|                                              |              |                 |                |
| **Robotic (10 rovers, supervised autonomy)** |              |                 |                |
| Capital (10 × $18,000)                       | $180,000     | —               | $180,000       |
| Operating (10 × $7,000/yr)                   | $70,000      | $70,000/yr      | $350,000       |
| Operator labor (supervised)                  | $10,000      | $10,000/yr      | $50,000        |
| Base station, infrastructure                 | $30,000      | $5,000/yr       | $50,000        |
| **Total**                                    | **$290,000** | **$85,000/yr**  | **$630,000**   |

**Payback period: 6–8 months of operation**

### 9.6 Sensitivity Analysis

The economic case depends on several variables. The following table shows break-even points:

| Variable           | Base Case | Break-Even | Notes                                         |
| ------------------ | --------- | ---------- | --------------------------------------------- |
| Operator ratio     | 1:10      | 1:4        | Below 1:4, labor savings disappear            |
| Snow events/season | 20        | 8          | Fewer events, longer payback                  |
| Clearing rate      | 0.5 mi/hr | 0.3 mi/hr  | Slower clearing extends labor hours           |
| Hardware cost      | $18,000   | $35,000    | Above $35k, 5-year TCO exceeds manual         |
| Rover lifespan     | 5 years   | 3 years    | Shorter life, capital cost amortizes worse    |
| Downtime rate      | 10%       | 40%        | Above 40% downtime, coverage fails            |
| LTE reliability    | 99%       | 95%        | Below 95%, operator intervention too frequent |

**Network reliability note**: The system is designed to fail safe on connectivity loss—rovers stop and hold position until communication is restored. Temporary outages (seconds to minutes) do not require operator intervention. Sustained outages (>5 minutes) require the operator to assess whether to wait, dispatch manual backup, or recover the unit. LTE reliability in urban/suburban areas typically exceeds 99% availability; rural or fringe coverage areas may require site surveys before deployment.

### 9.7 What This Analysis Excludes

**Excluded costs (favorable to robotic):**

- Injury and liability costs for manual labor
- Management overhead for seasonal hiring/training
- Political cost of inconsistent service

**Excluded costs (unfavorable to robotic):**

- Regulatory compliance (if new requirements emerge)
- Public relations and community acceptance
- Integration with existing municipal IT systems
- Vandalism and theft losses (mitigated by GPS tracking, remote disable, and low resale value of specialized components, but not zero)

**Excluded revenue:**

- Data value from mapped sidewalk conditions
- Year-round utilization (sweeping, inspection)

### 9.8 Capital vs. Operating Expense

Municipal budgets distinguish between capital expenditure (CapEx) and operating expenditure (OpEx). Robotic systems can be structured as either:

**CapEx model (purchase):**

- Upfront hardware purchase
- Annual maintenance contract
- Suits departments with capital budget availability
- Asset appears on municipal balance sheet

**OpEx model (lease/subscription):**

- Monthly or seasonal fee
- Includes hardware, software, maintenance
- Suits departments with operating budget flexibility
- No asset ownership; vendor retains title

**Robotics-as-a-Service (outcome-based):**

- Price per mile cleared
- Municipality pays for outcomes, not equipment
- Vendor assumes operational risk
- Requires vendor to operate fleet (not current model)

Current offering: CapEx (hardware sale) with optional annual software subscription.

### 9.9 Failure Cost Modeling

Robotic systems introduce failure modes that do not exist in manual labor. The cost of these failures must be modeled:

| Failure Mode           | Probability/Event | Impact                                       | Mitigation                        |
| ---------------------- | ----------------- | -------------------------------------------- | --------------------------------- |
| Unit breakdown (field) | 5–10%             | Route incomplete; redeploy spare or manual   | N+1 fleet sizing                  |
| Communication loss     | 1–2%              | Unit stops safely; resume when reconnected   | Failsafe firmware                 |
| Pedestrian incident    | Rare, high-impact | Liability, PR, potential program termination | Conservative speed limits, e-stop |
| Weather exceedance     | 2–5%              | Unit cannot operate; manual backup           | Clear operating envelope          |

**Note on pedestrian incidents**: No actuarial data exists for sidewalk robot incidents at scale. The system is designed to make such incidents unlikely through conservative speed limits (walking pace near pedestrians), active obstacle detection, and immediate stop capability. The impact of even a minor incident, however, is disproportionate to its probability—both in liability exposure and program continuity. See Section 7 for safety design details.

**Fleet sizing for reliability:**

To guarantee 95% coverage completion with 10% per-unit failure rate, deploy N+2 units (12 units for 10-unit workload).

### 9.10 Summary

Robotic sidewalk maintenance is economically viable under the following conditions:

1. Supervised autonomy (1:10 operator ratio) is achieved
2. Unit hardware cost remains below $25,000
3. Seasonal snow events exceed 10 per year
4. Municipality has 50+ miles of sidewalk (fleet amortization)
5. System uptime exceeds 85% during clearing events

At current capability (1:1 teleoperation), the system reduces injury risk and provides consistent coverage but does not reduce total cost. The economic case depends on autonomy development.

This analysis is intended to survive review by municipal finance staff. The numbers are conservative. Optimistic scenarios exist but are not presented here.

---

## 10. Governance, Data, and Vendor Risk

This section addresses questions that arise in procurement: Who owns what? What happens if the vendor fails? How do we exit?

### 10.1 Data Ownership

| Data Type                    | Owner             | Access                                   | Retention                                          |
| ---------------------------- | ----------------- | ---------------------------------------- | -------------------------------------------------- |
| Telemetry (position, status) | Customer          | Full export available                    | 90 days standard; extended on request              |
| Video recordings             | Customer          | Full export available                    | 30 days standard; incident footage retained longer |
| Route and map data           | Customer          | Full export available                    | Indefinite                                         |
| Aggregated fleet analytics   | Vendor            | Anonymized, used for product improvement | Indefinite                                         |
| Firmware and software        | Vendor (licensed) | Binary only; source escrow available     | N/A                                                |

Customers can export all operational data at any time in standard formats (CSV, JSON, GeoJSON). No data is held hostage.

### 10.2 Auditability

The system is designed for public accountability:

- **Open logs**: Clearing routes, times, and coverage are exportable and can be published
- **Incident reports**: Full documentation available for any flagged event
- **Performance metrics**: Uptime, coverage completion, response times tracked and reportable
- **Third-party audit**: System architecture and safety design available for independent review on request

Municipalities that publish open data can include robotic operations in their existing transparency frameworks.

### 10.3 Vendor Continuity Risk

What happens if the vendor (Muni) ceases operations?

**Hardware**: Rovers are owned by the customer. Hardware is based on commodity components (standard motors, batteries, compute modules). Third-party maintenance is feasible.

**Software**: Firmware source code is held in escrow with a third-party agent. In the event of vendor dissolution, escrow is released to customers with active support contracts.

**Documentation**: Full technical documentation (electrical schematics, mechanical drawings, firmware architecture) is included with enterprise purchases and available under escrow for smaller deployments.

**Transition period**: Vendor commits to 12 months notice before discontinuing support for any product generation, allowing customers to plan transitions.

### 10.4 Exit Strategy

Customers can exit the system at any time:

**Hardware disposition**:

- Resale to other operators (vendor will facilitate)
- Repurpose for other uses (platform is general-purpose)
- Dispose as electronic waste (no hazardous materials except batteries, which follow standard Li-ion disposal)

**Data migration**:

- All data exportable in standard formats
- No proprietary lock-in on operational data

**Contractual**:

- Annual software subscriptions can be cancelled with 30 days notice
- Hardware purchases have no ongoing obligation beyond optional maintenance contracts

### 10.5 Long-Term Vendor Relationship

The vendor's incentive is aligned with customer success:

- Subscription revenue depends on continued deployment
- Outcome-based pricing (future) depends on actual performance
- Reputation in a small market (municipal procurement) depends on references

The vendor is not optimizing for one-time hardware sales. The business model requires ongoing relationships with satisfied customers.

---

## 11. Conclusion

Municipal sidewalk maintenance is a constrained optimization problem. Labor is scarce, seasonal, and expensive. Equipment designed for roadways cannot operate on sidewalks. The result is a persistent service gap.

Robotic systems can close this gap under specific conditions:

1. The system operates reliably in the target environment (verified through pilot)
2. Supervised autonomy is achieved (one operator monitoring multiple units)
3. Total cost of ownership is lower than alternatives (validated in Section 9)
4. Safety and liability frameworks are acceptable to the deploying organization

The system described in this paper is designed to meet these conditions. It is operational today in pilot configuration. Specifications in this document reflect current capabilities, not roadmap projections.

Municipal robotics is viable only if treated as infrastructure: reliable, maintainable, accountable, and boring. This system is designed accordingly.

---

## Appendix A: Case Study — Lakewood, Ohio

This appendix applies the economic model to a specific municipality using publicly available data.

### A.1 City Profile

| Parameter                 | Value        | Source                    |
| ------------------------- | ------------ | ------------------------- |
| Population                | 49,517       | U.S. Census Bureau (2024) |
| Area                      | 5.5 sq mi    | Census                    |
| Population density        | ~9,000/sq mi | Highest in Ohio           |
| Sidewalk network          | 180+ miles   | City of Lakewood          |
| Street network            | 90 miles     | City of Lakewood          |
| Average snow events (1"+) | 24/season    | NOAA (Cleveland area)     |

Lakewood is a first-ring suburb of Cleveland, located on Lake Erie. It is the most densely populated city in Ohio and has been recognized as the state's most walkable city. The city does not provide school busing; students walk to school, making sidewalk accessibility a public safety issue.

### A.2 Current Sidewalk Snow Removal Approach

**Legal framework**: Lakewood Codified Ordinance 521.06 requires property owners to clear sidewalks within 24 hours after snowfall ends (by 9:00 a.m. in business districts).

**Enforcement**: Division of Housing and Building handles complaints. Residents report uncleared sidewalks via phone (216-529-6270) or online form.

**Assistance programs**: LakewoodAlive operates a volunteer snow removal program for seniors and residents with disabilities, dispatched when snowfall exceeds 3 inches.

**Municipal clearing**: None. The city clears streets but not sidewalks.

**Known issues**:

- Uneven compliance, particularly on residential segments
- Equity concerns (elderly, disabled, low-income residents cannot comply)
- Enforcement is reactive, not proactive
- No school busing creates safety dependency on sidewalk clearing

### A.3 Opportunity: Priority Route Clearing

Rather than attempting to clear all 180 miles, a robotic system would focus on a **priority network** where municipal control provides the highest value:

| Route Category                 | Miles  | Rationale                           |
| ------------------------------ | ------ | ----------------------------------- |
| School walking routes          | 25     | Student safety, liability reduction |
| Commercial districts           | 12     | Economic activity, accessibility    |
| Transit stops and corridors    | 8      | Public transit accessibility        |
| Senior/disabled housing access | 5      | Equity, ADA compliance              |
| **Priority network total**     | **50** |                                     |

This represents 28% of the total network but covers the highest-liability and highest-visibility segments.

### A.4 Fleet Sizing

For 50 miles of priority sidewalk with 24 snow events per season:

| Parameter                  | Value     | Calculation               |
| -------------------------- | --------- | ------------------------- |
| Miles to clear per event   | 50        | Priority network          |
| Clearing rate              | 0.5 mi/hr | System specification      |
| Rover-hours per event      | 100       | 50 ÷ 0.5                  |
| Target clearing time       | 10 hours  | Overnight + morning shift |
| Rovers required (parallel) | 10        | 100 ÷ 10                  |
| Redundancy (N+2)           | 12        | For reliability           |

**Recommended fleet**: 12 rovers (10 active, 2 spare)

### A.5 Cost Comparison

**Option 1: Manual clearing (hypothetical)**

If Lakewood were to clear the priority network with municipal crews:

| Cost Element                                 | Annual         | Calculation     |
| -------------------------------------------- | -------------- | --------------- |
| Seasonal labor (24 events × 50 mi × $267/mi) | $320,400       | Per Section 9.1 |
| Equipment (walk-behind blowers, ATVs)        | $25,000        | Amortized       |
| Supervision and overhead                     | $40,000        | Estimated       |
| **Annual total**                             | **$385,400**   |                 |
| **5-year total**                             | **$1,927,000** |                 |

_Note: This is a hypothetical; Lakewood does not currently incur these costs._

**Option 2: Contractor service**

Based on Minneapolis pilot data ($3,500/mile/season for priority routes):

| Cost Element              | Annual         | Calculation           |
| ------------------------- | -------------- | --------------------- |
| Contract (50 mi × $3,500) | $175,000       | Minneapolis benchmark |
| City oversight            | $25,000        | Staff time            |
| **Annual total**          | **$200,000**   |                       |
| **5-year total**          | **$1,000,000** |                       |

**Option 3: Robotic system (supervised autonomy)**

| Cost Element                                          | Year 1       | Years 2–5       | 5-Year Total |
| ----------------------------------------------------- | ------------ | --------------- | ------------ |
| Capital (12 rovers × $18,000)                         | $216,000     | —               | $216,000     |
| Operating (12 × $7,500/yr)                            | $90,000      | $90,000/yr      | $450,000     |
| Operator labor (1:10 ratio, 24 events × 10 hrs × $25) | $6,000       | $6,000/yr       | $30,000      |
| Base station and infrastructure                       | $35,000      | $5,000/yr       | $55,000      |
| **Annual total**                                      | **$347,000** | **$101,000/yr** |              |
| **5-year total**                                      |              |                 | **$751,000** |

**Option 4: Robotic system (current 1:1 teleop)**

| Cost Element                                          | Year 1       | Years 2–5       | 5-Year Total   |
| ----------------------------------------------------- | ------------ | --------------- | -------------- |
| Capital (12 rovers × $18,000)                         | $216,000     | —               | $216,000       |
| Operating (12 × $7,500/yr)                            | $90,000      | $90,000/yr      | $450,000       |
| Operator labor (1:1 ratio, 100 hrs × $25 × 24 events) | $60,000      | $60,000/yr      | $300,000       |
| Base station and infrastructure                       | $35,000      | $5,000/yr       | $55,000        |
| **Annual total**                                      | **$401,000** | **$155,000/yr** |                |
| **5-year total**                                      |              |                 | **$1,021,000** |

### A.6 Comparison Summary

| Approach                      | 5-Year TCO | vs. Contractor |
| ----------------------------- | ---------- | -------------- |
| Manual (hypothetical)         | $1,927,000 | +93%           |
| Contractor                    | $1,000,000 | baseline       |
| Robotic (1:1 teleop)          | $1,021,000 | +2%            |
| Robotic (supervised autonomy) | $751,000   | −25%           |

**Key findings**:

1. At current capability (1:1 teleop), robotic systems are cost-competitive with contractors but do not provide significant savings.

2. At supervised autonomy (1:10), robotic systems reduce 5-year TCO by approximately 25% compared to contractors.

3. Both robotic options are significantly cheaper than hypothetical manual municipal clearing.

4. The economic case for Lakewood depends on achieving supervised autonomy.

### A.7 Non-Economic Considerations

Beyond cost, robotic clearing offers Lakewood several advantages over the current property-owner mandate:

| Factor                    | Current (mandate)              | Robotic (municipal)       |
| ------------------------- | ------------------------------ | ------------------------- |
| Coverage consistency      | Uneven                         | Complete                  |
| Enforcement burden        | High                           | None                      |
| Equity (elderly/disabled) | Poor                           | Full service              |
| Liability                 | Distributed to property owners | Centralized, insured      |
| Data/verification         | None                           | Full telemetry            |
| Response time             | Variable (0–24+ hrs)           | Controlled (target 8 hrs) |
| School route priority     | No control                     | Guaranteed                |

For a city with no school busing and the state's highest population density, consistent sidewalk clearing is a public safety issue. The current mandate approach cannot guarantee consistency.

### A.8 Recommended Pilot

**Scope**: 5–10 miles of school walking routes in one quadrant of the city

**Duration**: One full winter season (December–March)

**Fleet**: 3 rovers (2 active, 1 spare)

**Capital cost**: ~$55,000

**Operating cost**: ~$25,000 for the season

**Evaluation criteria**:

- Coverage completion rate (target: 95%+)
- Clearing time (target: 8 hours from snowfall end)
- Uptime during events (target: 85%+)
- Incident rate (target: zero)
- Resident feedback (survey)

**Decision point**: If pilot succeeds, expand to full priority network (50 miles) in Year 2.

---

_Document version: Draft 2_
_Last updated: December 2024_
_Status: Internal review_
