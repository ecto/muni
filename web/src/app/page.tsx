import Link from "next/link";
import { Footer, FloatingHeader } from "@/components/layout";
import { PostCover } from "@/components/blog/PostCover";
import { StageStrip } from "@/components/home/StageStrip";
import { PeriodicTable } from "@/components/home/PeriodicTable";
import { formatDate, posts } from "@/lib/posts";
import "@/styles/post.css";
import "@/styles/home.css";

// The homepage is a trailer for the master plan: its cover, the climb, the stack, and three ways in.
export default function HomePage() {
  const plan = posts.find((p) => p.slug === "master-plan");

  return (
    <div className="post-page home-page">
      <FloatingHeader />

      <PostCover title="Robots that build robots" subtitle="What would it take for a machine to make the next one?">
        <div className="post-byline home-cta">
          <Link className="home-btn is-primary" href="/blog/master-plan">
            Read the plan
          </Link>
          <a className="pdf" href="/blog/master-plan.pdf">
            <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M6 1v7M3 5.5 6 8.5 9 5.5M2 11h8" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" /></svg>
            PDF
          </a>
        </div>
      </PostCover>

      <section className="home-section">
        <div className="home-head">
          <span className="home-eyebrow">The climb</span>
          <h2>Eight stages, one scale.</h2>
          <p>Each stage is a step up in scale, and each is built by the one before. Only the first is a plan; the rest are goals, and they are the point.</p>
        </div>
        <StageStrip />
      </section>

      <section className="home-section">
        <div className="home-head">
          <span className="home-eyebrow">The stack</span>
          <h2>One table, built in order.</h2>
          <p>Every project, software and hardware, is one element: Make turns source into parts, World simulates them, Body moves, Power feeds and thinks, Mind learns, Silicon is the far future.</p>
        </div>
        <PeriodicTable />
      </section>

      <section className="home-section">
        <div className="home-head">
          <span className="home-eyebrow">Get involved</span>
          <h2>Three ways in.</h2>
        </div>
        <div className="doors">
          <a className="door" href="mailto:info@muni.works?subject=One%20of%20the%20first%20ten%20limbs">
            <span className="door-title">Buy one of the first ten limbs</span>
            <span className="door-desc">A six-axis arm that ships with its measured gap. $5,000 list.</span>
          </a>
          <a className="door" href="mailto:info@muni.works?subject=Our%20city">
            <span className="door-title">Bring us your city</span>
            <span className="door-desc">A one-city, one-task pilot: litter first, sidewalks after.</span>
          </a>
          <a className="door" href="https://github.com/ecto/muni" target="_blank" rel="noopener noreferrer">
            <span className="door-title">Build with us</span>
            <span className="door-desc">Everything that can be open is open. Start with the plan's source.</span>
          </a>
        </div>
        {plan && (
          <div className="home-latest">
            <span className="home-eyebrow">Latest</span>
            {posts.slice(0, 3).map((p) => (
              <Link key={p.slug} href={`/blog/${p.slug}`} className="home-post">
                <time dateTime={p.date}>{formatDate(p.date)}</time>
                <span>{p.title}</span>
              </Link>
            ))}
          </div>
        )}
      </section>

      <div className="landing-footer">
        <div className="landing-container">
          <Footer />
        </div>
      </div>
    </div>
  );
}
