#import "../lib/template.typ": *
#import "../lib/diagrams.typ": *

// Sensor Mast Section
// Pole assembly, LiDAR, Camera

= Sensor Mast

The sensor mast is the rover's eyes. It elevates the LiDAR and 360° camera above the chassis to get an unobstructed view of the world.

Height matters. Too low and the sensors see mostly wheels. Too high and the mast becomes a sail in the wind and a lever arm for tip-overs. We settled on 700mm total height (from ground to camera) as a good compromise for sidewalk-scale operation.

The mast is intentionally simple: a tube, a clamp, and some brackets. If it gets bent (it will), you can straighten it or replace it in minutes.

= Sensor Mast Assembly

#procedure([Build sensor pole], time: "20 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Side view of mast
    // Frame
    rect((-3, 0), (3, 0.5), fill: diagram-light, stroke: 1pt + diagram-black)
    content((0, 0.25), text(size: 6pt)[Frame top])

    // Pole mount bracket
    rect((-0.5, 0.5), (0.5, 1.2), fill: diagram-gray, stroke: 1pt + diagram-black)
    callout-leader((0, 0.85), (-2, 0.85), "1")

    // Pole
    rect((-0.15, 1.2), (0.15, 5), fill: diagram-light, stroke: 1.5pt + diagram-black)
    callout-leader((0, 3), (-2, 3), "2")

    // LiDAR mount
    rect((-0.6, 4.2), (0.6, 4.8), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    content((0, 4.5), text(size: 5pt)[LiDAR])
    callout-leader((0, 4.5), (2, 4.5), "3")

    // Camera mount
    rect((-0.3, 5), (0.3, 5.4), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
    circle((0, 5.7), radius: 0.25, fill: diagram-black)
    content((1, 5.7), text(size: 6pt)[Camera])
    callout-leader((0, 5.7), (2, 5.7), "4")

    // Dimensions
    dim-v(1.5, 0, 5.7, "500-700", offset: 0.5)
    dim-v(-1.5, 0.5, 4.5, "pole", offset: 0.3)
  }),
  caption: [Sensor mast with LiDAR below camera for unobstructed 360° view.],
)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    *Components:*
    #table(
      columns: (auto, 1fr),
      stroke: none,
      inset: 3pt,
      [#text(fill: muni-orange, weight: "bold")[1]], [Pole mount bracket],
      [#text(fill: muni-orange, weight: "bold")[2]], [Carbon fiber or aluminum tube],
      [#text(fill: muni-orange, weight: "bold")[3]], [LiDAR mount plate],
      [#text(fill: muni-orange, weight: "bold")[4]], [Camera mount (1/4-20)],
    )
  ],
  [
    *Pole Specifications:*
    - Diameter: 25-30mm OD
    - Material: Carbon fiber (light) or 6061-T6 aluminum
    - Length: 400-600mm depending on design
    - Wall thickness: 2mm minimum
  ]
)

#pagebreak()

// =============================================================================

= LiDAR Mounting

#procedure([Mount LiDAR sensor], time: "15 min", difficulty: 2)

#v(1em)

#grid(
  columns: (1fr, 1fr),
  column-gutter: 2em,
  [
    #figure(
      cetz.canvas({
        import cetz.draw: *

        // TOP VIEW
        content((0, 4), text(size: 8pt, weight: "bold")[TOP VIEW])

        // LiDAR body (circle)
        circle((0, 1.5), radius: 1.8, fill: diagram-light, stroke: 1.5pt + diagram-black)
        content((0, 1.5), text(size: 8pt, weight: "bold")[Mid-360])

        // Mounting holes (4 corners)
        for (dx, dy) in ((1, 1), (-1, 1), (-1, -1), (1, -1)) {
          circle((dx * 0.9, 1.5 + dy * 0.9), radius: 0.12, fill: white, stroke: 0.5pt + diagram-black)
        }

        // Front indicator
        motion-arrow((0, 3.3), (0, 3.9))
        content((0.6, 3.6), text(size: 5pt)[Front])

        // 360° FOV indicator (dashed circle)
        circle((0, 1.5), radius: 2.6, stroke: (paint: muni-orange, dash: "dashed", thickness: 1pt))
        content((0, -1.6), text(size: 6pt, fill: muni-orange)[360° FOV])
      }),
      caption: [Top view: full 360° sensing],
    )
  ],
  [
    #figure(
      cetz.canvas({
        import cetz.draw: *

        // SIDE VIEW
        content((0, 4), text(size: 8pt, weight: "bold")[SIDE VIEW])

        // LiDAR body
        rect((-2, 2), (2, 3.2), fill: diagram-light, stroke: 1.5pt + diagram-black, radius: 2pt)
        content((0, 2.6), text(size: 8pt, weight: "bold")[Mid-360])

        // Connector
        rect((2, 2.3), (2.7, 2.9), fill: diagram-gray, stroke: 0.5pt + diagram-black)
        content((3.3, 2.6), text(size: 5pt)[Cable])

        // Mount bracket
        rect((-1.3, 1.3), (1.3, 2), fill: diagram-gray, stroke: 1pt + diagram-black)
        content((0, 1.65), text(size: 5pt)[Bracket])

        // Pole
        rect((-0.3, -0.8), (0.3, 1.3), fill: diagram-light, stroke: 1pt + diagram-black)
        content((0, -1.2), text(size: 6pt)[Pole])
      }),
      caption: [Side view: cable routes down],
    )
  ],
)

#v(1em)

*Installation:*

+ Attach LiDAR to mount plate with M3 bolts
+ Level the mount plate (use spirit level)
+ Secure mount plate to pole with hose clamps or bolts
+ Route cable inside pole or along outside with ties
+ Connect to Jetson via Ethernet

*Orientation:*
- LiDAR "front" should face rover front
- Ensure level within ±1°
- No obstructions in 360° view

#pagebreak()

// =============================================================================

= Camera Mounting

#procedure([Mount 360° camera], time: "10 min", difficulty: 1)

#v(1em)

#figure(
  cetz.canvas({
    import cetz.draw: *

    // Camera (sphere style)
    circle((0, 2), radius: 0.8, fill: diagram-light, stroke: 1.5pt + diagram-black)
    circle((0, 2.8), radius: 0.25, fill: diagram-black)
    circle((0, 1.2), radius: 0.25, fill: diagram-black)
    content((0, 2), text(size: 6pt)[X4])
    content((1.5, 2.8), text(size: 6pt)[Lens (front)])
    content((1.5, 1.2), text(size: 6pt)[Lens (rear)])

    // 1/4-20 mount
    line((0, 0.4), (0, 1.2), stroke: 1.5pt + diagram-black)
    content((1, 0.7), text(size: 6pt)[1/4-20 mount])

    // Mount adapter
    rect((-0.4, 0), (0.4, 0.4), fill: diagram-gray, stroke: 1pt + diagram-black)
    content((0, 0.2), text(size: 5pt)[Adapter])

    // Pole top
    rect((-0.2, -1.5), (0.2, 0), fill: diagram-light, stroke: 1pt + diagram-black)

    // FOV indicator
    line((0, 2), (-2, 3.5), stroke: 0.5pt + muni-orange)
    line((0, 2), (2, 3.5), stroke: 0.5pt + muni-orange)
    line((0, 2), (-2, 0.5), stroke: 0.5pt + muni-orange)
    line((0, 2), (2, 0.5), stroke: 0.5pt + muni-orange)
    content((2.5, 2), text(size: 6pt, fill: muni-orange)[360° × 180°])
  }),
  caption: [Camera at mast top. Dual lenses capture full spherical view.],
)

#v(1em)

*Mount Options:*
- 1/4-20 threaded insert in pole top cap
- GoPro-style mount adapter
- Custom 3D-printed adapter

*Cable Routing:*
- USB-C cable to Jetson
- Route inside pole if possible
- Secure with cable ties
- Leave strain relief loop at camera

#v(1em)

*Camera Settings:*
#spec-table(
  [*Setting*], [*Value*],
  [Mode], [Live streaming (H.265)],
  [Resolution], [4K or 5.7K],
  [Frame rate], [30 fps],
  [Stabilization], [FlowState (on)],
)

#pagebreak()
