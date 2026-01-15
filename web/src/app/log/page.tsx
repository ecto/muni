import type { Metadata } from "next";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";

export const metadata: Metadata = {
  title: "Engineering Log",
  description: "Engineering updates and progress from Municipal Robotics.",
};

export default function LogPage() {
  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content">
          <Card title="ATTACHMENT MCU FIRMWARE">
            <Pre>
              <span className="entry-date">2025-12-28</span>
{`

Good progress on the ESP32 firmware for tool attachments.
Got the OLED display working with a status UI and serial
command interface. LED strip driver is ready but needs a
level shifter (in the mail).

Also consolidated session recording to single files, added
a sessions browser to the fleet UI, and documented the
sensor mast design.`}
            </Pre>
          </Card>

          <Card title="BVR0 COMPLETE">
            <Pre>
              <span className="entry-date">2025-12</span>
{`

Base platform engineering prototype finished. 4-wheel
skid-steer rover with Jetson Orin NX, proving out the
mechanical and electrical architecture. bvr1 R&D and
supervised autonomy coming in Artifact residency.

Total build cost: ~$5,000 in parts. Assembly time: ~40 hours.
All designs, firmware, and documentation published to GitHub.`}
            </Pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
