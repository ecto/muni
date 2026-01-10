import type { Metadata } from "next";
import Link from "next/link";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card } from "@/components/ui";

export const metadata: Metadata = {
  title: "Documentation",
  description: "Technical documentation for Municipal Robotics autonomous sidewalk rovers.",
};

export default function DocsPage() {
  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content">
          <Card title="TECHNICAL MANUALS" highlight>
            <pre>
              <a href="/docs/bvr0-manual.pdf">BVR0 Technical Manual</a>
{`     Assembly, wiring, operation, safety
`}
              <a href="/docs/bvr0-datasheet.pdf">BVR0 Datasheet</a>
{`            Specifications at a glance
`}
              <Link href="/viewer">CAD Viewer</Link>
{`                Interactive 3D model viewer`}
            </pre>
          </Card>

          <Card title="ARCHITECTURE">
            <pre>
{`┌─────────────────────────────────────────────────────────────────┐
│                         Operator Station                         │
│  ┌────────────┐  ┌────────────┐  ┌────────────────────────────┐ │
│  │ Video View │  │ Xbox Ctrl  │  │ Telemetry                  │ │
│  │ (H.265)    │  │ (gamepad)  │  │ (voltage, temps, mode)     │ │
│  └────────────┘  └────────────┘  └────────────────────────────┘ │
└────────────────────────────┬────────────────────────────────────┘
                             │ QUIC / UDP
              ┌──────────────┴──────────────┐
              │       Cloud Relay           │
              │  (optional, for NAT)        │
              └──────────────┬──────────────┘
                             │ LTE
┌────────────────────────────┴────────────────────────────────────┐
│                            Rover                                 │
│  ┌────────────────────────────────────────────────────────────┐ │
│  │  bvrd (Jetson Orin NX)                                     │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌───────────────┐ │ │
│  │  │ teleop  │  │ control │  │ state   │  │ tools         │ │ │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └───────┬───────┘ │ │
│  │       └────────────┴────────────┴───────────────┘         │ │
│  │                          │ CAN bus                         │ │
│  └──────────────────────────┼─────────────────────────────────┘ │
│       ┌─────────┬───────────┼───────────┬─────────┐             │
│  ┌────┴────┐ ┌──┴───┐  ┌────┴────┐ ┌────┴───┐ ┌───┴────┐       │
│  │ VESC FL │ │ FR   │  │ RL      │ │ RR     │ │ Tool   │       │
│  └────┬────┘ └──┬───┘  └────┬────┘ └────┬───┘ └───┬────┘       │
│       │         │           │           │         │             │
│  ┌────┴────┐ ┌──┴───┐  ┌────┴────┐ ┌────┴───┐ ┌───┴────┐       │
│  │ Motor   │ │ Motor│  │ Motor   │ │ Motor  │ │ Auger  │       │
│  └─────────┘ └──────┘  └─────────┘ └────────┘ └────────┘       │
└─────────────────────────────────────────────────────────────────┘`}
            </pre>
          </Card>

          <Card title="SOFTWARE">
            <pre>
              <strong>Crates</strong>
{`
├─ types       Shared types (Twist, Mode, etc.)
├─ can         CAN bus + VESC protocol
├─ control     Differential drive mixer, rate limiter, watchdog
├─ state       State machine (Disabled → Idle → Teleop → EStop)
├─ hal         GPIO, ADC, power monitoring
├─ teleop      LTE communications, command/telemetry
├─ tools       Tool discovery and implementations
├─ recording   Session recording to Rerun .rrd files
├─ metrics     Real-time metrics push to Depot (InfluxDB UDP)
├─ gps         GPS receiver (NMEA parsing)
└─ camera      Camera capture and video streaming

`}
              <strong>Binaries</strong>
{`
├─ bvrd        Main daemon (runs on Jetson)
└─ cli         Debug/control CLI tool`}
            </pre>
          </Card>

          <Card title="HARDWARE REFERENCE">
            <pre>
              <strong>Specifications</strong>
{`
├─ Compute       Jetson Orin NX
├─ Chassis       24×24" 2020 aluminum extrusion
├─ Motors        4× hoverboard hub motors (350W each)
├─ ESCs          4× VESC (CAN bus, 500K baud)
├─ Power         48V main, 12V accessory rail
└─ Connectivity  LTE + WiFi mesh

`}
              <strong>Guides</strong>
{`
├─ `}
              <a href="https://github.com/ecto/muni/blob/main/docs/hardware/bom.md">Bill of Materials</a>
{`
├─ `}
              <a href="https://github.com/ecto/muni/blob/main/docs/hardware/motors.md">Motor Configuration</a>
{`
├─ `}
              <a href="https://github.com/ecto/muni/blob/main/docs/hardware/networking.md">Networking Setup</a>
{`
└─ `}
              <a href="https://github.com/ecto/muni/blob/main/docs/hardware/rtk.md">RTK GPS</a>
            </pre>
          </Card>

          <Card title="SAFETY" highlight>
            <pre>
              <strong>Watchdog</strong>
{`      No command for 250ms → safe stop
`}
              <strong>E-Stop</strong>
{`        Immediate stop, requires explicit release
`}
              <strong>Rate Limit</strong>
{`    Acceleration capped to prevent tip-over
`}
              <strong>Voltage</strong>
{`       Low battery → reduced power → shutdown

Three independent E-Stop paths:
├─ Physical button on rover
├─ Software command (spacebar)
└─ Connection loss timeout`}
            </pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
