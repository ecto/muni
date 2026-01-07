#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Maintenance Section
// Schedule, Troubleshooting, Storage

= Maintenance

A well-maintained rover is a reliable rover. The maintenance tasks are simple: clean it, inspect it, keep the bolts tight and the battery healthy.

The schedule below is based on real-world operation in Cleveland conditions (salt, snow, mud, temperature swings). If you operate in a milder environment, you can extend the intervals. If you're running daily in harsh conditions, shorten them.

= Maintenance Schedule

#procedure([Preventive maintenance overview], time: "varies", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Timeline
    line((-5.5, 0), (5.5, 0), stroke: 2pt + diagram-black)

    // Weekly
    circle((-4, 0), radius: 0.6, fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((-4, 1.2), text(size: 10pt, weight: "bold")[Weekly])
    content((-4, -1), text(size: 7pt)[Light clean])

    // Monthly
    circle((0, 0), radius: 0.6, fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((0, 1.2), text(size: 10pt, weight: "bold")[Monthly])
    content((0, -1), text(size: 7pt)[Inspect])

    // Seasonal
    circle((4, 0), radius: 0.6, fill: diagram-light, stroke: 1.5pt + diagram-black)
    content((4, 1.2), text(size: 10pt, weight: "bold")[Seasonal])
    content((4, -1), text(size: 7pt)[Full service])
  }),
  caption: none,
)

#v(2em)

#grid(
  columns: (1fr, 1fr, 1fr),
  column-gutter: 1em,
  [
    *Weekly*
    - Clean wheels and chassis
    - Wipe camera lens
    - Wipe LiDAR lens
    - Check connector seating
    - Verify wheel spin
    - Test E-Stop function
  ],
  [
    *Monthly*
    - Inspect all wiring
    - Check bolt torque
    - Clean electrical contacts
    - Check battery health
    - Update firmware
    - Review error logs
  ],
  [
    *Seasonal*
    - Full electrical inspection
    - Check wheel bearings
    - Replace worn tires
    - Deep clean chassis
    - Calibrate sensors
    - Battery capacity test
  ]
)

#pagebreak()

// =============================================================================

= Troubleshooting

#procedure([Diagnose common issues], time: "varies", difficulty: 2)

#v(1em)

#spec-table(
  [*Symptom*], [*Likely Cause*], [*Solution*],
  [Won't power on], [Battery disconnect], [Check XT90 connection, verify fuse],
  [No video feed], [Camera USB], [Reconnect camera, check USB hub power],
  [Motor not responding], [CAN wiring], [Check CAN connections, verify VESC ID],
  [Erratic movement], [VESC ID mismatch], [Verify IDs match wheel positions],
  [E-Stop won't release], [Button stuck], [Check relay wiring, verify mechanism],
  [Overheating], [Ventilation blocked], [Clean vents, reduce load],
  [Poor LTE signal], [Antenna position], [Reposition antenna, check SIM],
  [Battery dies quickly], [Battery age], [Check cell balance, replace if needed],
  [Jerky motion], [Motor calibration], [Re-run VESC motor detection],
  [Drift to one side], [Wheel alignment], [Re-align motor brackets],
)

#v(1em)

*Diagnostic Commands:*

```bash
# Check system status
bvr status

# List CAN devices
bvr can scan

# Test individual motor
bvr motor test <id>

# View recent logs
journalctl -u bvrd -n 100
```

#v(0.5em)

#video-link("https://muni.works/docs/troubleshooting", [Troubleshooting Walkthrough])

#pagebreak()

// =============================================================================

= Storage

#procedure([Store rover properly], time: "5 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    let items = (
      ("50-60%", (-4, 0), "Battery charge"),
      ("Disconnect", (0, 0), "Unplug battery"),
      ("15-25°C", (4, 0), "Temperature"),
    )

    for (icon, pos, label) in items {
      rect((pos.at(0) - 1.3, -0.7), (pos.at(0) + 1.3, 0.7),
           fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 4pt)
      content(pos, text(size: 11pt, weight: "bold")[#icon])
      content((pos.at(0), -1.3), text(size: 8pt)[#label])
    }
  }),
  caption: none,
)

#v(2em)

*Short-Term Storage (< 1 week):*

#checklist(
  [Press E-Stop],
  [Disconnect battery],
  [Cover if stored outdoors],
)

#v(1em)

*Long-Term Storage (> 1 week):*

#checklist(
  [Charge battery to 50-60%],
  [Disconnect battery completely],
  [Clean chassis and wheels],
  [Cover camera and LiDAR lenses],
  [Store in dry location (15-25°C)],
  [Check battery monthly (recharge if < 40%)],
)

#v(1em)

*Returning from Storage:*

+ Inspect for moisture, corrosion, pest damage
+ Charge battery fully
+ Run pre-flight checklist
+ Test all functions before field use

#pagebreak()
