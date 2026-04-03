# BVR Suspension System: Mathematical Analysis

Should BVR have a suspension system? This document works through the
relevant physics from first principles using BVR's actual parameters.

## BVR Physical Parameters (from bvr.toml / bvr1-dimensions.md)

| Parameter | Value |
|-----------|-------|
| Mass (m) | ~20 kg |
| Track width (W) | 0.55 m |
| Wheelbase (L) | 0.55 m |
| Wheel diameter | 0.165 m (solid hub motor, no pneumatic tire) |
| CG height (H_cg) | 0.189 m |
| Ground clearance | 0.075 m |
| Max speed | 3.0 m/s |
| Drive type | 4-wheel skid-steer (differential drive) |

## Mathematical Domains Involved

Suspension analysis draws from several areas of applied math and mechanics:

1. **Kinematics / constraint analysis** - geometry of wheel-ground contact
2. **Statics** - normal force distribution, traction budgets
3. **Rigid body dynamics** - how the chassis responds to terrain inputs
4. **Vibration theory** - spring-mass-damper transmissibility, natural frequencies
5. **Impact mechanics** - impulse loads from curbs and cracks
6. **Terrain spectral analysis** - power spectral density of surface roughness
7. **Control theory** - how variable traction affects skid-steer controllability

We'll work through each.

---

## 1. The Overconstrained Contact Problem

This is the central geometric argument for suspension on a 4-wheeled vehicle.

### Three points define a plane

A rigid body resting on a surface has 6 degrees of freedom (x, y, z, roll,
pitch, yaw). Ground contact through wheels provides normal-force constraints.
Three contact points fully constrain the vertical DOFs (z, roll, pitch),
leaving the body free to translate and yaw on the surface.

A fourth contact point is *redundant*: it can only be satisfied if all four
wheels happen to be coplanar. On a perfectly flat surface, this is trivially
true. On real terrain, it is not.

### When does a wheel lift?

Consider BVR's roughly square footprint (W = L = 0.55 m). Label the wheels:

```
  FL ──────── FR
  │            │
  │     CG     │       W = 0.55 m
  │            │
  RL ──────── RR

  ◄──── L ────►
        0.55 m
```

If FL encounters a bump of height h while the other three wheels are on a
flat plane, the rigid frame must pivot. The pivot axis passes through the
two wheels adjacent to FL (i.e., FR and RL). The opposite wheel (RR) moves
in the opposite vertical direction.

For a rectangular footprint, the perpendicular distance from each diagonal
corner to the FR-RL line is:

```
d = (W × L) / sqrt(W² + L²)
```

For a square (W = L = 0.55):

```
d = (0.55 × 0.55) / sqrt(0.55² + 0.55²)
   = 0.3025 / 0.7778
   = 0.389 m
```

Since FL and RR are equidistant from the FR-RL diagonal, the lift at RR
equals the bump height at FL:

```
Δz_RR = h × (d_RR / d_FL) = h × 1.0 = h
```

**Result**: On a square-footprint rigid frame, a bump h at one corner
causes the diagonal corner to lift by exactly h.

### How much bump before a wheel lifts?

With solid hub-motor wheels (no pneumatic compliance), the only deformation
is the hard rubber/polyurethane tire contact patch. Estimated compliance:
~1-2 mm under 5 kg load. So the effective "suspension travel" from tire
deformation alone is approximately:

```
z_compliance ≈ 1-2 mm
```

Any terrain irregularity exceeding ~2 mm across the wheelbase diagonal
(0.78 m) will cause one wheel to lose ground contact.

For reference, typical sidewalk surface irregularities:
- Expansion joint offsets: 3-15 mm
- Frost heave steps: 5-30 mm
- Tree root lifting: 10-50 mm
- Snow/ice buildup: 5-50 mm (variable)
- Cracked concrete lips: 2-10 mm

**Conclusion**: On real sidewalks, BVR will be in 3-point contact most
of the time. The fourth wheel will frequently be unloaded or lifted.

---

## 2. Normal Force Distribution (Statics)

### Flat ground (ideal case)

On a flat surface, with the CG centered, each wheel carries:

```
N_per_wheel = mg / 4 = (20 × 9.8) / 4 = 49 N
```

### 3-point contact (real terrain)

When one wheel lifts off, the remaining three wheels form a triangle.
The normal forces are determined by static equilibrium (sum of forces
and moments = 0). For a symmetric CG position:

```
Loaded wheels: three wheels share mg = 196 N
Unloaded wheel: N = 0
```

The distribution among the three loaded wheels depends on CG position
relative to the support triangle. With the CG centered, each loaded
wheel carries approximately:

```
N_loaded ≈ mg / 3 ≈ 65.3 N
```

### Tilted terrain (slope angle alpha)

On a cross-slope of angle alpha, the CG shifts laterally. The normal
force on the downhill wheels increases while uphill wheels decrease:

```
N_uphill = (mg/2) × (1 - (H_cg × tan(α)) / (W/2))
N_downhill = (mg/2) × (1 + (H_cg × tan(α)) / (W/2))
```

For BVR on a steep curb cut (alpha = 8°):

```
N_uphill = 98 × (1 - (0.189 × 0.14) / 0.275) = 98 × (1 - 0.096) = 88.6 N  (per side)
N_downhill = 98 × (1 + 0.096) = 107.4 N  (per side)
```

Combined with the 3-point contact problem, one uphill wheel could already
be unloaded from terrain irregularity, leaving the other uphill wheel
at ~88.6 N while two downhill wheels share ~107.4 N. The traction budget
becomes asymmetric.

---

## 3. Traction Impact for Skid-Steer

This is where suspension (or its absence) matters most for BVR.

### Why skid-steer is sensitive to wheel unloading

In a differential/skid-steer robot, turning requires lateral scrubbing
forces at the wheels. Each side of the robot must generate both:
- Longitudinal force (driving forward/backward)
- Lateral friction (resisting or generating scrub)

The available friction at each wheel is bounded by the Coulomb friction
cone:

```
sqrt(F_x² + F_y²) ≤ μ × N
```

If a wheel is unloaded (N = 0), it contributes zero friction in any
direction. This has two effects:

1. **Reduced turning authority**: The unloaded side can generate less
   lateral scrub force, causing yaw drift or sluggish turning.
2. **Unpredictable yaw**: As the robot rolls over uneven terrain, the
   traction distribution shifts suddenly (wheel lifts/lands), causing
   jerky or oscillatory yaw behavior.

### Quantifying the turning impact

BVR's skid-steer turning moment is:

```
M_turn = (F_right - F_left) × W/2
```

With all 4 wheels loaded equally on icy terrain (mu = 0.25):

```
F_side = 2 × μ × N = 2 × 0.25 × 49 = 24.5 N  per side
M_turn_max = 2 × 24.5 × 0.275 = 13.5 N·m
```

With one wheel unloaded (say RL), the left side has only one wheel:

```
F_left = 1 × μ × 65.3 = 16.3 N
F_right = 2 × μ × 65.3 = 32.7 N
```

The total friction is unchanged (mu × mg), but the max yaw moment
depends on which side lost the wheel:

```
M_turn_max = (32.7 - 16.3) × 0.275 = 4.5 N·m   (turning toward unloaded side)
M_turn_max = (32.7 + 16.3) × 0.275 = 13.5 N·m   (turning away from unloaded side)
```

**Result**: Losing one wheel cuts maximum turning moment in the
unfavorable direction by ~67%. On ice, this can mean inability to
execute a commanded turn.

---

## 4. Vibration Analysis

### The spring-mass-damper model

A suspension system is fundamentally a mass-spring-damper between the
chassis (sprung mass) and the wheel (unsprung mass). The simplest
single-DOF model:

```
m × z'' + c × z' + k × z = c × z_road' + k × z_road
```

where:
- m = sprung mass (chassis share per wheel)
- k = spring stiffness (N/m)
- c = damping coefficient (N·s/m)
- z = chassis vertical displacement
- z_road = road surface input

### Natural frequency

The undamped natural frequency:

```
f_n = (1/2π) × sqrt(k/m)
```

For comfortable ride quality (vehicles): f_n ≈ 1-2 Hz.
For a utility robot where "comfort" isn't relevant, the goal is instead
to keep all wheels in ground contact. The design target is:

```
f_n < f_terrain
```

where f_terrain is the dominant terrain excitation frequency.

### Terrain excitation frequency

At speed v over periodic bumps spaced lambda apart:

```
f_terrain = v / lambda
```

For BVR at 2 m/s over sidewalk joints spaced every 1.5 m:

```
f_terrain = 2.0 / 1.5 = 1.33 Hz
```

For random roughness at 2 m/s, the dominant energy is typically at 1-5 Hz.

### Transmissibility

The ratio of chassis motion to road input:

```
T(f) = sqrt((1 + (2ξr)²) / ((1 - r²)² + (2ξr)²))
```

where r = f/f_n and xi is the damping ratio (c / 2×sqrt(km)).

Key behaviors:
- r << 1 (low freq): T ≈ 1 (chassis follows road exactly -- good for contact)
- r = 1 (resonance): T >> 1 (amplification -- bad, causes bouncing)
- r >> 1 (high freq): T → 0 (isolation -- chassis ignores bumps)

Without suspension (rigid), T = 1 at all frequencies: every bump is
transmitted fully to the chassis. There's no resonance, but also no
isolation and no compliance for maintaining contact.

### What would BVR suspension look like?

For a 20 kg robot with 4 suspension points, each corner supports ~5 kg.
To get f_n ≈ 3 Hz (faster response than a car, prioritizing ground contact
over isolation):

```
k = m × (2π × f_n)² = 5 × (2π × 3)² = 5 × 355 = 1,776 N/m
```

That's a relatively soft spring (about 1.8 kN/m). For reference, a
typical rubber bushing or short compression spring can easily provide this.

Static deflection under load:

```
δ_static = mg/k = (5 × 9.8) / 1776 = 27.6 mm
```

So ~28 mm of suspension travel would be needed, with a total stroke of
perhaps 40-50 mm to handle dynamic loads. This is very achievable with
simple mechanisms.

Critical damping coefficient:

```
c_crit = 2 × sqrt(k × m) = 2 × sqrt(1776 × 5) = 188 N·s/m
```

A damping ratio of xi ≈ 0.3-0.5 (underdamped, common for vehicles) gives:

```
c = ξ × c_crit = 0.4 × 188 = 75 N·s/m
```

---

## 5. Impact Mechanics (Curbs and Steps)

### Impulse from hitting a curb edge

When a wheel hits a step of height h at velocity v, the impulse depends
on the wheel radius R and step height:

```
Contact angle: θ = acos(1 - h/R)
Horizontal impulse: J = m_wheel × v × sin(θ)
Vertical impulse: J_v = m_wheel × v × cos(θ) × tan(θ)
```

For BVR hitting a 15 mm sidewalk lip at 2 m/s:

```
θ = acos(1 - 0.015/0.0825) = acos(0.818) = 35.1°
```

The wheel (hub motor, ~1 kg) experiences:

```
J ≈ 1.0 × 2.0 × sin(35.1°) = 1.15 N·s
```

The resulting force depends on impact duration. For rigid wheel on
rigid edge, contact time is very short (order of 1 ms), giving:

```
F_peak ≈ J / Δt = 1.15 / 0.001 = 1,150 N
```

This is ~6x the static wheel load (49 N per wheel). These impulse loads
propagate directly into the chassis on a rigid frame, stressing
fasteners and electronics.

With a suspension spring (k = 1,776 N/m), the impact is absorbed over a
longer time:

```
Δt_spring ≈ π × sqrt(m/k) = π × sqrt(5/1776) = 0.167 s
F_peak_spring ≈ J / Δt_spring = 1.15 / 0.167 = 6.9 N
```

**Result**: Suspension reduces peak impact forces by roughly two orders
of magnitude (from ~1,150 N to ~7 N for this scenario). This
significantly reduces mechanical fatigue and protects electronics.

---

## 6. Terrain Power Spectral Density

Road and sidewalk roughness is typically characterized by a power spectral
density (PSD) of the surface elevation profile:

```
S(Ω) = S_0 × (Ω / Ω_0)^(-w)
```

where:
- Omega = spatial frequency (cycles/m)
- S_0 = roughness coefficient (m³/cycle)
- w ≈ 2-3 (spectral exponent, typically 2.5 for roads)

ISO 8608 classifies road roughness into categories A-H. Sidewalks
typically fall in class B-C (slightly rougher than smooth highway).

For class B-C at BVR speeds (1-3 m/s), the RMS vertical acceleration
of the unsprung mass is:

```
a_rms = (2π)² × v × sqrt(∫ S(Ω) × Ω² dΩ)
```

Numerical estimates for class C roughness at 2 m/s give a_rms ≈ 0.5-2 m/s²
on the chassis. This is not "uncomfortable" (BVR has no passengers) but
causes continuous vibration-induced micro-slips at the wheel-ground interface,
reducing effective traction by an estimated 5-15%.

---

## 7. Control Theory: Effect on Skid-Steer Odometry

BVR currently estimates motion from wheel encoder (VESC ERPM) data. The
differential drive model assumes all wheels maintain ground contact:

```
v = (v_R + v_L) / 2        (linear velocity)
ω = (v_R - v_L) / W        (angular velocity)
```

When a wheel lifts, its encoder reads free-spinning velocity (or zero if
still in contact with reduced load). This corrupts the odometry estimate.

The firmware blends IMU yaw with wheel odometry (`imu_yaw_weight` in
config). This partially mitigates the problem, but the linear velocity
estimate remains degraded when wheels lose contact.

With suspension maintaining all-wheel contact, the encoder signals remain
valid, improving dead-reckoning accuracy. The expected improvement depends
on terrain roughness but is estimated at 10-30% reduction in odometry drift
for typical sidewalk conditions.

---

## 8. Simple Suspension Architectures

Given BVR's constraints (small, light, 4WD skid-steer), here are the
main options with their mathematical models:

### Option A: Pivot axle (one axle free to rock)

```
  FL ──────── FR        ← Fixed axle
  │            │
  │     CG     │
  │            │
  RL ─── ⊕ ── RR       ← Pivot axle (rocks about centerline)
```

- DOF added: 1 (rear axle roll)
- Contact guarantee: 4 wheels always in contact on terrain with single-axis
  curvature
- Math: eliminates the 3-point contact problem for roll-axis terrain
  variation; pitch-axis variation still causes diagonal lift
- Complexity: low (single pivot bearing)
- Limitation: does not help with per-wheel bumps or pitch-axis irregularities

### Option B: Rocker-bogie (NASA-style)

```
       ┌─ FL
  ────●┤                ← Rocker arm with differential
       └─ RL
```

Each side has a rocker linking front and rear wheels, connected through a
differential (mechanical linkage or passive averaging bar). This provides:

- DOF added: 2 (one rocker per side)
- Contact guarantee: all wheels maintain contact on arbitrary terrain
  (up to mechanism travel limits)
- Math: the differential ensures that chassis pitch = average of both
  rocker angles; each wheel independently follows terrain
- Complexity: moderate (rocker arms, central differential or averaging bar)
- Limitation: more complex mechanism, adds width

### Option C: Independent spring mounts (simplest)

```
  FL ╤══════╤ FR        ╤ = spring + linear guide
  │  ║      ║  │
  │  ║  CG  ║  │
  │  ║      ║  │
  RL ╤══════╤ RR
```

Each wheel is mounted on a short linear spring (compression spring, rubber
bushing, or elastomer block). This is the simplest to implement:

- DOF added: 4 (one vertical DOF per wheel)
- Contact guarantee: excellent, each wheel independently tracks terrain
- Math: 4 independent spring-mass-damper systems (see section 4)
- Spring rate per corner: k ≈ 1,776 N/m (for f_n = 3 Hz)
- Travel needed: ~40-50 mm
- Complexity: low (spring + linear guide or trailing arm per wheel)
- Limitation: chassis can pitch and roll more freely (may need damping
  to prevent wallowing)

### Option D: Compliant wheel mounts (minimal)

Replace rigid L-bracket wheel mounts with elastomer bushings or add
rubber isolation pads. Provides 2-5 mm of compliance:

- DOF added: ~4 (soft, limited travel)
- Contact improvement: marginal (2-5 mm compliance vs 3-15 mm bumps)
- Math: very stiff springs (k >> 10,000 N/m), high natural frequency
- Complexity: minimal (bushing swap)
- Limitation: insufficient travel for most sidewalk irregularities

---

## 9. Decision Matrix

| Factor | No Suspension | Pivot Axle | Independent Springs |
|--------|:---:|:---:|:---:|
| All-wheel contact on sidewalk | poor | fair | good |
| Turning on uneven terrain | poor | fair | good |
| Impact protection | none | partial | good |
| Odometry accuracy | fair | fair | good |
| Mechanical complexity | none | low | moderate |
| Mass added | 0 | ~0.3 kg | ~0.5-1 kg |
| Risk of mechanism failure | none | low | low-moderate |
| Traction on ice + bumps | poor | fair | good |

---

## 10. Quantitative Recommendation

### Is suspension mathematically necessary?

The answer depends on the operating envelope:

**On smooth, well-maintained sidewalks** (irregularities < 3 mm): No.
The rigid frame maintains adequate 4-point contact, and the traction
budget is sufficient. Most of the time, BVR operates fine without
suspension.

**On typical winter sidewalks** (irregularities 5-30 mm from frost heave,
ice buildup, plowed snow ridges): Yes, it would help significantly.
The math shows:

1. One wheel lifts on any diagonal terrain variation > ~2 mm (section 1)
2. This cuts max turning moment by up to 67% in the unfavorable direction
   (section 3)
3. Impact forces from sidewalk lips are ~6x static wheel load, stressing
   the frame (section 5)
4. Odometry degrades 10-30% from wheel contact loss (section 7)

### Minimum viable suspension

The simplest option that solves the core problem (ground conformity):

**Independent spring mounts (Option C)**, with:
- k ≈ 1,800 N/m per corner (f_n ≈ 3 Hz)
- 40-50 mm travel
- Light damping (xi ≈ 0.3-0.5, c ≈ 55-95 N·s/m)
- Static deflection ~28 mm

This can be implemented with compression springs on linear guides, or
trailing-arm linkages with elastomer springs. Total added mass: ~0.5-1 kg.

### When to defer

If BVR is currently operating only on clear, dry sidewalks and the
primary failure modes are software-related (not traction-related), then
suspension is a second-order improvement. The mathematical case becomes
compelling when:

- Operating on winter surfaces (ice + irregular terrain)
- Observing yaw control issues during teleop on rough terrain
- Seeing elevated vibration-induced failures in electronics/fasteners
- Odometry drift is a bottleneck for autonomous navigation

---

## Summary of Math Used

| Domain | Key Equations | What It Tells Us |
|--------|---------------|------------------|
| Planar geometry | d = WL/sqrt(W²+L²); Δz = h | When the 4th wheel lifts |
| Statics | N = mg/n; F = μN | How traction distributes |
| Friction cones | sqrt(Fx²+Fy²) ≤ μN | Turning authority per wheel |
| Spring-mass-damper | f_n = sqrt(k/m)/2π | Suspension natural frequency |
| Transmissibility | T(f) = ... | Vibration isolation ratio |
| Impact mechanics | J = mv·sin(θ); F = J/Δt | Peak forces from curb hits |
| Terrain PSD | S(Ω) = S_0(Ω/Ω_0)^(-w) | Statistical surface roughness |
| Odometry kinematics | v = (v_R+v_L)/2; ω = (v_R-v_L)/W | Encoder accuracy vs contact |

These span undergraduate-level dynamics, vibration theory, and vehicle
dynamics. Nothing here requires graduate-level math; the challenge is in
applying these models with realistic terrain data, which comes from field
measurements rather than theory.
