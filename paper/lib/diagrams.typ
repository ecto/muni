// Technical Illustration Library
// IKEA-style diagram primitives for BVR documentation

#import "@preview/cetz:0.3.4"

// =============================================================================
// COLOR PALETTE
// =============================================================================

#let diagram-black = rgb("#1a1a1a")
#let diagram-gray = rgb("#666666")
#let diagram-light = rgb("#e5e5e5")
#let diagram-accent = rgb("#E86A33")  // Muni orange
#let diagram-danger = rgb("#dc2626")
#let diagram-success = rgb("#16a34a")

// =============================================================================
// ISOMETRIC PROJECTION
// =============================================================================

// Convert 3D coordinates to 2D isometric projection
// Standard isometric: 30° from horizontal
#let iso(x, y, z) = {
  let angle = 30deg
  let cos-a = calc.cos(angle)
  let sin-a = calc.sin(angle)
  (
    x * cos-a - y * cos-a,
    x * sin-a + y * sin-a + z
  )
}

// Isometric scale factor (objects appear ~81% of actual size)
#let iso-scale = 0.8165

// Draw an isometric box (rectangular prism)
#let iso-box(ctx, origin, size, fill: none, stroke: 1pt + black, label: none) = {
  import cetz.draw: *
  
  let (ox, oy, oz) = origin
  let (sx, sy, sz) = size
  
  // 8 corners
  let p0 = iso(ox, oy, oz)
  let p1 = iso(ox + sx, oy, oz)
  let p2 = iso(ox + sx, oy + sy, oz)
  let p3 = iso(ox, oy + sy, oz)
  let p4 = iso(ox, oy, oz + sz)
  let p5 = iso(ox + sx, oy, oz + sz)
  let p6 = iso(ox + sx, oy + sy, oz + sz)
  let p7 = iso(ox, oy + sy, oz + sz)
  
  // Draw visible faces (top, right, front)
  // Top face
  line(p4, p5, p6, p7, close: true, fill: fill, stroke: stroke)
  // Right face
  line(p1, p2, p6, p5, close: true, fill: fill, stroke: stroke)
  // Front face
  line(p0, p1, p5, p4, close: true, fill: fill, stroke: stroke)
  
  // Hidden edges (dashed)
  line(p0, p3, stroke: (dash: "dashed", paint: gray, thickness: 0.5pt))
  line(p3, p2, stroke: (dash: "dashed", paint: gray, thickness: 0.5pt))
  line(p3, p7, stroke: (dash: "dashed", paint: gray, thickness: 0.5pt))
  
  if label != none {
    let center = iso(ox + sx/2, oy + sy/2, oz + sz + 0.3)
    content(center, label)
  }
}

// =============================================================================
// CALLOUTS AND ANNOTATIONS
// =============================================================================

// Numbered callout bubble
#let callout(pos, number, size: 0.4) = {
  import cetz.draw: *
  circle(pos, radius: size, fill: diagram-accent, stroke: none)
  content(pos, text(fill: white, weight: "bold", size: 9pt)[#number])
}

// Leader line with callout
#let callout-leader(from, to, number, text-content: none, anchor: "left") = {
  import cetz.draw: *
  
  // Line from point to callout
  line(from, to, stroke: 0.75pt + diagram-gray)
  
  // Callout bubble
  circle(to, radius: 0.35, fill: diagram-accent, stroke: none)
  content(to, text(fill: white, weight: "bold", size: 8pt)[#number])
  
  // Optional text label
  if text-content != none {
    let offset = if anchor == "left" { (-0.6, 0) } else { (0.6, 0) }
    let label-pos = (to.at(0) + offset.at(0), to.at(1) + offset.at(1))
    content(label-pos, text(size: 7pt)[#text-content], anchor: anchor)
  }
}

// Simple label with line
#let label-line(from, to, label, anchor: "south") = {
  import cetz.draw: *
  line(from, to, stroke: 0.5pt + diagram-gray)
  content(to, text(size: 7pt)[#label], anchor: anchor)
}

// =============================================================================
// MOTION AND ACTION ARROWS
// =============================================================================

// Curved motion arrow (for rotation)
#let rotation-arrow(center, radius, start-angle, end-angle, stroke-color: diagram-black) = {
  import cetz.draw: *
  arc(center, start: start-angle, stop: end-angle, radius: radius, 
      stroke: 1.5pt + stroke-color, mark: (end: ">"))
}

// Straight motion arrow
#let motion-arrow(from, to, label: none, stroke-color: diagram-black) = {
  import cetz.draw: *
  line(from, to, stroke: 1.5pt + stroke-color, mark: (end: ">"))
  if label != none {
    let mid = ((from.at(0) + to.at(0)) / 2, (from.at(1) + to.at(1)) / 2 + 0.3)
    content(mid, text(size: 7pt)[#label])
  }
}

// Insert/push action arrow
#let insert-arrow(from, to) = {
  import cetz.draw: *
  line(from, to, stroke: (thickness: 2pt, paint: diagram-accent, dash: "solid"), 
       mark: (end: ">", fill: diagram-accent))
}

// =============================================================================
// DIMENSION LINES
// =============================================================================

// Horizontal dimension
#let dim-h(y, x1, x2, label, offset: 0.3) = {
  import cetz.draw: *
  let y-line = y - offset
  
  // Extension lines
  line((x1, y), (x1, y-line - 0.1), stroke: 0.5pt + diagram-gray)
  line((x2, y), (x2, y-line - 0.1), stroke: 0.5pt + diagram-gray)
  
  // Dimension line with arrows
  line((x1, y-line), (x2, y-line), stroke: 0.5pt + diagram-black,
       mark: (start: "|", end: "|"))
  
  // Label
  content(((x1 + x2) / 2, y-line - 0.25), text(size: 7pt)[#label])
}

// Vertical dimension
#let dim-v(x, y1, y2, label, offset: 0.3) = {
  import cetz.draw: *
  let x-line = x + offset
  
  // Extension lines
  line((x, y1), (x-line + 0.1, y1), stroke: 0.5pt + diagram-gray)
  line((x, y2), (x-line + 0.1, y2), stroke: 0.5pt + diagram-gray)
  
  // Dimension line
  line((x-line, y1), (x-line, y2), stroke: 0.5pt + diagram-black,
       mark: (start: "|", end: "|"))
  
  // Label (rotated)
  content((x-line + 0.35, (y1 + y2) / 2), text(size: 7pt)[#label])
}

// =============================================================================
// PART PRIMITIVES - TOP VIEW (2D)
// =============================================================================

// Hub motor wheel (top view)
#let wheel-top(pos, size: 0.6, label: none) = {
  import cetz.draw: *
  let (x, y) = pos
  rect((x - size/2, y - size*1.5), (x + size/2, y + size*1.5), 
       fill: diagram-black, radius: 2pt)
  if label != none {
    content((x, y - size*1.5 - 0.3), text(size: 6pt)[#label])
  }
}

// Hub motor wheel (side view / cross section)
#let wheel-side(pos, radius: 1, tire-width: 0.3) = {
  import cetz.draw: *
  let (x, y) = pos
  
  // Tire (outer)
  circle((x, y), radius: radius, stroke: 2pt + diagram-black, fill: diagram-light)
  // Hub (inner)
  circle((x, y), radius: radius * 0.6, stroke: 1pt + diagram-black, fill: white)
  // Axle
  circle((x, y), radius: radius * 0.15, fill: diagram-black)
}

// VESC motor controller (top view)
#let vesc-top(pos, size: (1.2, 0.8), id: none) = {
  import cetz.draw: *
  let (x, y) = pos
  let (w, h) = size
  
  rect((x - w/2, y - h/2), (x + w/2, y + h/2), 
       fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
  
  // Heat sink fins
  for i in range(5) {
    let fx = x - w/2 + 0.15 + i * (w - 0.3) / 4
    line((fx, y - h/2), (fx, y - h/2 - 0.1), stroke: 0.5pt + diagram-gray)
  }
  
  if id != none {
    content((x, y), text(size: 6pt, weight: "bold")[VESC #id])
  }
}

// Jetson compute module (top view)
#let jetson-top(pos, size: (2, 1.5)) = {
  import cetz.draw: *
  let (x, y) = pos
  let (w, h) = size
  
  rect((x - w/2, y - h/2), (x + w/2, y + h/2),
       fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
  
  // Heatsink pattern
  rect((x - w/2 + 0.1, y - h/2 + 0.1), (x + w/2 - 0.1, y + h/2 - 0.1),
       stroke: 0.5pt + diagram-gray)
  
  content((x, y), text(size: 6pt)[Jetson])
}

// Battery pack (top view)
#let battery-top(pos, size: (2.5, 1.5)) = {
  import cetz.draw: *
  let (x, y) = pos
  let (w, h) = size
  
  rect((x - w/2, y - h/2), (x + w/2, y + h/2),
       fill: diagram-accent, stroke: 1pt + diagram-black, radius: 4pt)
  
  // Terminal indicators
  circle((x + w/2 - 0.2, y + 0.2), radius: 0.1, fill: diagram-danger, stroke: none)
  circle((x + w/2 - 0.2, y - 0.2), radius: 0.1, fill: diagram-black, stroke: none)
  
  content((x, y), text(size: 7pt, fill: white, weight: "bold")[48V 20Ah])
}

// 2020 aluminum extrusion cross-section
#let extrusion-section(pos, size: 0.4) = {
  import cetz.draw: *
  let (x, y) = pos
  let s = size
  
  // Outer square
  rect((x - s/2, y - s/2), (x + s/2, y + s/2), stroke: 1pt + diagram-black)
  
  // T-slot grooves (simplified)
  let groove = s * 0.15
  rect((x - groove, y + s/2 - groove), (x + groove, y + s/2), fill: white, stroke: 0.5pt + diagram-gray)
  rect((x - groove, y - s/2), (x + groove, y - s/2 + groove), fill: white, stroke: 0.5pt + diagram-gray)
  rect((x - s/2, y - groove), (x - s/2 + groove, y + groove), fill: white, stroke: 0.5pt + diagram-gray)
  rect((x + s/2 - groove, y - groove), (x + s/2, y + groove), fill: white, stroke: 0.5pt + diagram-gray)
  
  // Center hole
  circle((x, y), radius: s * 0.12, fill: white, stroke: 0.5pt + diagram-gray)
}

// LiDAR sensor (Livox Mid-360)
#let lidar-top(pos, size: 0.8) = {
  import cetz.draw: *
  let (x, y) = pos
  
  // Cylindrical body (shown as circle from top)
  circle((x, y), radius: size, fill: diagram-light, stroke: 1pt + diagram-black)
  
  // Dome
  circle((x, y), radius: size * 0.7, stroke: 0.5pt + diagram-gray)
  
  // FOV indicator arcs
  for angle in range(0, 360, step: 45) {
    let a = angle * 1deg
    let inner = size * 0.8
    let outer = size * 1.2
    line(
      (x + inner * calc.cos(a), y + inner * calc.sin(a)),
      (x + outer * calc.cos(a), y + outer * calc.sin(a)),
      stroke: 0.3pt + diagram-gray
    )
  }
}

// Camera (Insta360)
#let camera-top(pos, radius: 0.3) = {
  import cetz.draw: *
  let (x, y) = pos
  
  // Body
  circle((x, y), radius: radius, fill: diagram-black, stroke: none)
  
  // Lens indicators (dual fisheye)
  circle((x, y + radius * 0.4), radius: radius * 0.25, fill: diagram-gray, stroke: none)
  circle((x, y - radius * 0.4), radius: radius * 0.25, fill: diagram-gray, stroke: none)
}

// =============================================================================
// PART PRIMITIVES - ISOMETRIC (3D)
// =============================================================================

// Isometric wheel/hub motor
#let wheel-iso(origin, radius: 0.8, width: 0.4) = {
  import cetz.draw: *
  let (ox, oy, oz) = origin
  
  // Simplified as cylinder approximation
  let p1 = iso(ox, oy - width/2, oz)
  let p2 = iso(ox, oy + width/2, oz)
  
  // Draw ellipses for wheel faces
  circle(p1, radius: radius * iso-scale, fill: diagram-light, stroke: 1pt + diagram-black)
  circle(p2, radius: radius * iso-scale, fill: diagram-black, stroke: 1pt + diagram-black)
  
  // Connecting lines (top and bottom of cylinder)
  let top1 = (p1.at(0), p1.at(1) + radius * iso-scale)
  let top2 = (p2.at(0), p2.at(1) + radius * iso-scale)
  let bot1 = (p1.at(0), p1.at(1) - radius * iso-scale)
  let bot2 = (p2.at(0), p2.at(1) - radius * iso-scale)
  
  line(top1, top2, stroke: 1pt + diagram-black)
  line(bot1, bot2, stroke: 1pt + diagram-black)
}

// =============================================================================
// CONNECTOR SYMBOLS
// =============================================================================

// XT connector (XT90, XT60, XT30)
#let connector-xt(pos, size: "90", orientation: "h") = {
  import cetz.draw: *
  let (x, y) = pos
  let w = if size == "90" { 0.8 } else if size == "60" { 0.6 } else { 0.4 }
  let h = w * 0.6
  
  if orientation == "h" {
    rect((x - w/2, y - h/2), (x + w/2, y + h/2), fill: rgb("#f5d742"), stroke: 1pt + diagram-black, radius: 2pt)
    content((x, y), text(size: 5pt, weight: "bold")[XT#size])
  } else {
    rect((x - h/2, y - w/2), (x + h/2, y + w/2), fill: rgb("#f5d742"), stroke: 1pt + diagram-black, radius: 2pt)
    content((x, y), text(size: 5pt, weight: "bold")[XT#size])
  }
}

// Deutsch DT connector
#let connector-dt(pos, pins: 6) = {
  import cetz.draw: *
  let (x, y) = pos
  let w = 0.8
  let h = 0.5
  
  rect((x - w/2, y - h/2), (x + w/2, y + h/2), fill: diagram-gray, stroke: 1pt + diagram-black, radius: 2pt)
  
  // Pin holes
  let pin-spacing = (w - 0.2) / (pins - 1)
  for i in range(pins) {
    let px = x - w/2 + 0.1 + i * pin-spacing
    circle((px, y), radius: 0.05, fill: diagram-black, stroke: none)
  }
}

// =============================================================================
// SAFETY SYMBOLS
// =============================================================================

// Warning triangle
#let warning-symbol(pos, size: 0.5) = {
  import cetz.draw: *
  let (x, y) = pos
  let h = size * 0.866  // equilateral triangle height
  
  line(
    (x, y + h * 0.67),
    (x - size/2, y - h * 0.33),
    (x + size/2, y - h * 0.33),
    close: true,
    fill: rgb("#fbbf24"),
    stroke: 1.5pt + diagram-black
  )
  content((x, y), text(size: 10pt, weight: "bold")[!])
}

// Danger/prohibition circle
#let danger-symbol(pos, size: 0.4) = {
  import cetz.draw: *
  let (x, y) = pos
  
  circle((x, y), radius: size, fill: none, stroke: 2pt + diagram-danger)
  line((x - size * 0.7, y + size * 0.7), (x + size * 0.7, y - size * 0.7), stroke: 2pt + diagram-danger)
}

// E-stop button symbol
#let estop-symbol(pos, size: 0.6) = {
  import cetz.draw: *
  let (x, y) = pos
  
  // Outer ring
  circle((x, y), radius: size, fill: diagram-danger, stroke: 2pt + diagram-black)
  // Inner button
  circle((x, y), radius: size * 0.6, fill: rgb("#991b1b"), stroke: 1pt + diagram-black)
  // STOP text
  content((x, y), text(fill: white, size: 6pt, weight: "bold")[STOP])
}

// =============================================================================
// FLOW DIAGRAM HELPERS
// =============================================================================

// Process box
#let process-box(pos, label, width: 2, height: 0.8, fill-color: diagram-light) = {
  import cetz.draw: *
  let (x, y) = pos
  
  rect((x - width/2, y - height/2), (x + width/2, y + height/2),
       fill: fill-color, stroke: 1pt + diagram-black, radius: 4pt)
  content((x, y), text(size: 7pt)[#label])
}

// Decision diamond
#let decision-box(pos, label, size: 1) = {
  import cetz.draw: *
  let (x, y) = pos
  let s = size / 2
  
  line((x, y + s), (x + s, y), (x, y - s), (x - s, y), close: true,
       fill: diagram-light, stroke: 1pt + diagram-black)
  content((x, y), text(size: 6pt)[#label])
}

// Flow arrow
#let flow-arrow(from, to) = {
  import cetz.draw: *
  line(from, to, stroke: 1pt + diagram-black, mark: (end: ">"))
}

// =============================================================================
// COMPLETE ROVER DIAGRAMS
// =============================================================================

// BVR0 top view (complete assembly)
#let rover-top-view(scale: 1) = {
  import cetz.draw: *
  
  let s = scale
  
  // Chassis frame
  rect((-3 * s, -3 * s), (3 * s, 3 * s), stroke: 1.5pt + diagram-black, radius: 4pt)
  
  // Wheels at corners
  for (x, y, label) in ((-3, 2.5, "FL"), (3, 2.5, "FR"), (-3, -2.5, "RL"), (3, -2.5, "RR")) {
    rect(((x - 0.5) * s, (y - 0.8) * s), ((x + 0.5) * s, (y + 0.8) * s), 
         fill: diagram-black, radius: 2pt)
    content((x * s, (y - 1.2) * s), text(size: 6pt)[#label])
  }
  
  // Electronics area
  rect((-2 * s, -2 * s), (2 * s, 1 * s), fill: diagram-light, stroke: 0.5pt + diagram-gray, radius: 2pt)
  content((0, -0.5 * s), text(size: 7pt)[Electronics])
  
  // Tool mount (front)
  rect((-1.5 * s, 2.5 * s), (1.5 * s, 3 * s), fill: diagram-light, stroke: 0.5pt + diagram-gray)
  content((0, 2.75 * s), text(size: 6pt)[Tool Mount])
  
  // Sensor mast
  circle((0, 1.5 * s), radius: 0.3 * s, fill: diagram-black)
  content((0.8 * s, 1.5 * s), text(size: 6pt)[Sensors])
  
  // Direction indicator
  line((0, 3.5 * s), (0, 4.5 * s), stroke: 1pt + diagram-black, mark: (end: ">"))
  content((0, 4.8 * s), text(size: 7pt)[FRONT])
}

// BVR0 side view
#let rover-side-view(scale: 1) = {
  import cetz.draw: *
  
  let s = scale
  
  // Chassis
  rect((-3 * s, 0.5 * s), (3 * s, 1.5 * s), stroke: 1pt + diagram-black, radius: 2pt)
  content((0, 1 * s), text(size: 7pt)[Chassis])
  
  // Wheels
  circle((-2.5 * s, 0), radius: 0.6 * s, stroke: 1pt + diagram-black, fill: diagram-light)
  circle((2.5 * s, 0), radius: 0.6 * s, stroke: 1pt + diagram-black, fill: diagram-light)
  
  // Ground line
  line((-4 * s, -0.6 * s), (4 * s, -0.6 * s), stroke: 0.5pt + diagram-gray)
  
  // Sensor mast
  line((0, 1.5 * s), (0, 3.5 * s), stroke: 1.5pt + diagram-black)
  
  // LiDAR
  rect((-0.4 * s, 2.5 * s), (0.4 * s, 3 * s), fill: diagram-light, stroke: 1pt + diagram-black, radius: 2pt)
  content((1.2 * s, 2.75 * s), text(size: 6pt)[LiDAR])
  
  // Camera
  circle((0, 3.5 * s), radius: 0.25 * s, fill: diagram-black)
  content((1 * s, 3.5 * s), text(size: 6pt)[Camera])
}
