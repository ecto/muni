import type { Metadata } from "next";
import Link from "next/link";
import { Header, Footer } from "@/components/layout";
import { Card, Pre } from "@/components/ui";

export const metadata: Metadata = {
  title: "Order Cancelled",
  description: "Your order was cancelled. No payment was processed.",
};

export default function CancelPage() {
  return (
    <div className="page">
      <div className="container">
        <Header showSubtitle={false} />

        <main className="content">
          <Card title="ORDER CANCELLED">
            <Pre>
{`No payment was processed. Your card was not charged.

`}
              <strong>Still interested?</strong>
{`

`}
              <Link href="/products" className="cta">
                View products
              </Link>
{`

Questions? `}
              <a href="mailto:info@muni.works">info@muni.works</a>
            </Pre>
          </Card>
        </main>

        <Footer />
      </div>
    </div>
  );
}
