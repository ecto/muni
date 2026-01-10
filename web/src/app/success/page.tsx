import type { Metadata } from "next";
import Link from "next/link";
import { CheckCircle } from "@phosphor-icons/react/dist/ssr";
import { Header, Footer } from "@/components/layout";
import { Card } from "@/components/ui";

export const metadata: Metadata = {
  title: "Order Confirmed",
  description: "Your Muni preorder is confirmed.",
};

export default function SuccessPage() {
  return (
    <div className="page">
      <div className="container">
        <Header showSubtitle={false} />

        <main className="content">
          <Card
            title={
              <>
                <CheckCircle size={16} weight="regular" /> RESERVATION CONFIRMED
              </>
            }
            className="success-highlight"
          >
            <pre>
{`Your $500 deposit has been received and your order is secured.

`}
              <strong>What happens next:</strong>
{`

1. You'll receive an email receipt from Stripe immediately
2. Our team will contact you within 48 hours via email
3. We'll discuss deployment timeline and configuration
4. Balance payment due before shipment (Q2-Q3 2026)

`}
              <strong>Refund policy:</strong>
{`
Full refund available until production begins (Q2 2026).
Email `}
              <a href="mailto:info@muni.works">info@muni.works</a>
{` to request refund.`}
            </pre>
          </Card>

          <Card title="WHILE YOU WAIT">
            <pre>
{`Join the community and explore the platform:

  ◎ `}
              <a href="https://github.com/ecto/muni">Browse the source code on GitHub</a>
{`
  ◇ `}
              <Link href="/docs">Read technical documentation</Link>
{`
  ◈ `}
              <a href="https://github.com/ecto/muni/tree/main/bvr/docs/hardware">
                Review BVR0 build guide
              </a>
{`
  ◉ `}
              <a href="mailto:info@muni.works?subject=Pilot%20program">
                Ask about pilot programs
              </a>
{`

Questions? Email `}
              <a href="mailto:info@muni.works">info@muni.works</a>
            </pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
