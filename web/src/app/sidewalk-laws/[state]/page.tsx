import type { Metadata } from "next";
import Link from "next/link";
import { notFound } from "next/navigation";
import { Header, NavBar, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";
import { states, findStateBySlug } from "@/lib/data/states";
import "../sidewalk-laws.css";

interface PageProps {
  params: Promise<{ state: string }>;
}

export function generateStaticParams() {
  return states.map((state) => ({ state: state.slug }));
}

export async function generateMetadata({ params }: PageProps): Promise<Metadata> {
  const { state: stateSlug } = await params;
  const state = findStateBySlug(stateSlug);

  if (!state) {
    return {
      title: "State Not Found",
    };
  }

  return {
    title: `Sidewalk Snow Removal Laws in ${state.name}`,
    description: `${state.name} sidewalk liability laws, clearance requirements, and municipal responsibilities. ${state.liability.summary.slice(0, 120)}...`,
    alternates: {
      canonical: `https://muni.works/sidewalk-laws/${state.slug}`,
    },
  };
}

export default async function StateLawsPage({ params }: PageProps) {
  const { state: stateSlug } = await params;
  const state = findStateBySlug(stateSlug);

  if (!state) {
    notFound();
  }

  const liabilityStandardLabels = {
    "reasonable-care": "Reasonable Care Standard",
    "natural-accumulation": "Natural Accumulation Rule",
    varies: "Varies by Municipality",
  };

  return (
    <div className="page">
      <div className="container">
        <Header />
        <NavBar />

        <main className="content">
          <h1 className="state-page-title">Sidewalk Snow Removal Laws in {state.name}</h1>

          <Card title="OVERVIEW">
            <div className="state-overview">
              <div className="overview-stat">
                <span className="stat-value">{state.avgAnnualSnowfall}&quot;</span>
                <span className="stat-label">Avg Annual Snowfall</span>
              </div>
              <div className="overview-stat">
                <span className="stat-value">{state.liability.typicalClearanceWindow}</span>
                <span className="stat-label">Typical Clearance Window</span>
              </div>
              <div className="overview-stat">
                <span className="stat-value">
                  {state.liability.municipalResponsibility ? "Yes" : "Limited"}
                </span>
                <span className="stat-label">Municipal Responsibility</span>
              </div>
            </div>
          </Card>

          <Card title="LIABILITY STANDARD">
            <Pre>
              <strong>{liabilityStandardLabels[state.liability.standard]}</strong>
{`

${state.liability.summary}

`}
              <a href={state.liability.source} target="_blank" rel="noopener noreferrer">
                View source legislation →
              </a>
            </Pre>
          </Card>

          <Card title="MAJOR CITIES">
            <Pre>
{`Cities in ${state.name} with significant sidewalk infrastructure:

`}
              {state.majorCities.map((city, i) => (
                <span key={city}>
                  {i === state.majorCities.length - 1 ? "└─ " : "├─ "}
                  {city}
                  {"\n"}
                </span>
              ))}
{`
Each municipality may have additional local ordinances
beyond state-level requirements.`}
            </Pre>
          </Card>

          <Card title="REDUCE LIABILITY WITH AUTOMATION" highlight>
            <Pre>
              <strong>Autonomous rovers address key liability concerns:</strong>
{`

  • Consistent 24/7 clearing during active snowfall
  • Documented operations for legal protection
  • No labor dependency during shortages
  • Faster response times than manual crews

Average sidewalk clearing cost savings: 90%+
Typical payback period: <1 season

`}
              <Link href="/products" className="cta-inline">
                Learn about BVR1 →
              </Link>
              {" "}
              <Link href="/about#pilot" className="cta-inline-secondary">
                Join pilot program
              </Link>
            </Pre>
          </Card>

          <Card title="OTHER STATES">
            <div className="other-states">
              {states
                .filter((s) => s.slug !== state.slug)
                .map((s) => (
                  <Link key={s.slug} href={`/sidewalk-laws/${s.slug}`} className="other-state-link">
                    {s.abbreviation}
                  </Link>
                ))}
            </div>
            <Pre>
{`
`}
              <Link href="/sidewalk-laws">← View all states</Link>
            </Pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
