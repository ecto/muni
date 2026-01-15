import type { Metadata } from "next";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";

export const metadata: Metadata = {
  title: "About",
  description: "Municipal Robotics builds autonomous rovers for sidewalk maintenance.",
};

export default function AboutPage() {
  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content">
          <Card title="WHY WE BUILD">
            <Pre>
{`We watched people (parents with strollers, elderly neighbors,
kids walking to school) forced into the street because there
was no safe path on the sidewalk.

Sidewalks are public infrastructure. They deserve the same
attention we give to roads.

So we asked a simple question: why are we still clearing
sidewalks by hand? Why isn't there a better way?

We're building that future in the open, one rover at a time.`}
            </Pre>
          </Card>

          <Card title="THE COMPANY">
            <Pre>
              <strong>Municipal Robotics</strong>
{` was founded in 2025 in Cleveland, Ohio.

We're a small team of engineers who believe public spaces
should work for everyone. We build in the Midwest because
that's where winters are real and the problem is acute.

Everything we build is open source. Check the code, build
your own, or buy from us. We don't hide behind patents.`}
            </Pre>
          </Card>

          <Card title="TIMELINE">
            <Pre>
{`  2025-12  `}
              <span className="status-complete">■</span>
{` bvr0 engineering prototype complete
           │   Base platform proving out architecture
           │
  2026-01  `}
              <span className="status-dev">◐</span>
{` F.Inc Artifact program (San Francisco)
           │   bvr1 R&D, supervised autonomy, production unit
           │
  2026-Q3  ○ bvr1 shipping to pilot partners
           │   10 production units, Midwest municipalities
           │
  2027     ○ Fleet autonomy at scale
               Multi-unit deployments validated`}
            </Pre>
          </Card>

          <Card title="PILOT PROGRAM" highlight id="pilot">
            <Pre>
{`We're looking for 3-5 partners in the Midwest to deploy
bvr1 fleets in winter 2026. Pilot partners get:

  ├─ Discounted hardware pricing
  ├─ On-site deployment and training
  ├─ Direct engineering support
  └─ Input on product roadmap

`}
              <strong>Ideal partners:</strong>
{`
  • Cities with 10-50 miles of sidewalk
  • Existing public works team
  • Willingness to iterate and provide feedback
  • Universities and research labs also welcome

`}
              <a className="cta" href="mailto:info@muni.works?subject=Pilot%20Program%20Application">
                Apply for the pilot program →
              </a>
            </Pre>
          </Card>

          <Card title="TEAM">
            <Pre>
              <strong>Cam Pedersen</strong>
{`, Founder
Autonomous vehicle scheduling at Uber. CTO and co-founder
at DitchCarbon.

Built bvr0 from scratch in Cleveland: mechanical design,
electrical, firmware, autonomous navigation software.

`}
              <a href="mailto:info@muni.works">info@muni.works</a>
            </Pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
