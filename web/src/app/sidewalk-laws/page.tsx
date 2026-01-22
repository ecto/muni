import type { Metadata } from "next";
import Link from "next/link";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";
import { states } from "@/lib/data/states";
import "./sidewalk-laws.css";

export const metadata: Metadata = {
  title: "Sidewalk Snow Removal Laws by State",
  description:
    "State-by-state guide to sidewalk snow removal liability, clearance requirements, and municipal responsibilities across the US snow belt.",
  alternates: {
    canonical: "https://muni.works/sidewalk-laws",
  },
};

export default function SidewalkLawsIndexPage() {
  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content">
          <h1 className="sr-only">Sidewalk Snow Removal Laws by State</h1>

          <Card title="SIDEWALK LIABILITY LAWS">
            <Pre>
{`Understanding sidewalk snow removal liability is critical for
municipalities, property managers, and facilities teams.

Each state has different standards for:
  • Who is responsible (municipality vs property owner)
  • Clearance timeframes after snowfall
  • Liability for slip-and-fall injuries
  • Penalties for non-compliance

Browse state-specific information below.`}
            </Pre>
          </Card>

          <Card title="SNOW BELT STATES">
            <div className="states-grid">
              {states.map((state) => (
                <Link
                  key={state.slug}
                  href={`/sidewalk-laws/${state.slug}`}
                  className="state-card"
                >
                  <span className="state-abbr">{state.abbreviation}</span>
                  <span className="state-name">{state.name}</span>
                  <span className="state-snow">
                    {state.avgAnnualSnowfall}&quot; avg snow/year
                  </span>
                </Link>
              ))}
            </div>
          </Card>

          <Card title="WHY THIS MATTERS">
            <Pre>
              <strong>Municipal liability exposure is significant:</strong>
{`

  • $35B+ spent annually on snow removal and liability costs
  • 1M+ slip-and-fall injuries on icy sidewalks per year
  • Lawsuits can cost municipalities $100K-$1M+ per incident

`}
              <strong>Autonomous rovers reduce liability:</strong>
{`

  • 24/7 operation clears sidewalks faster
  • Consistent coverage eliminates missed areas
  • Documented operations provide liability protection
  • Zero labor dependency during labor shortages

`}
              <Link href="/products" className="cta">
                See how Muni rovers can help →
              </Link>
            </Pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
