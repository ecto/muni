import Link from "next/link";
import { FloatingHeader } from "@/components/layout";
import { PostCover } from "@/components/blog/PostCover";
import "@/styles/post.css";
import "@/styles/home.css";

// One frame. What matters now is the first revenue: frog's users becoming kyte's customers.
// The plan is one click away; everything else lives in it.
const mail = (subject: string) => `mailto:info@muni.works?subject=${encodeURIComponent(subject)}`;

export default function HomePage() {
  return (
    <div className="post-page home-page home-one">
      <FloatingHeader />
      <PostCover title="Robots that build robots" subtitle="An autonomous company, amplified by intelligence.">
        <p className="home-pitch">
          It starts with <span className="name n-mind">frog</span>, a free coding agent that runs on your Mac.
        </p>
        <div className="post-byline home-cta">
          <a className="home-btn is-primary" href={mail("frog early access")}>Get frog</a>
          <Link className="home-btn is-quiet" href="/blog/master-plan">Read the plan</Link>
          <a className="pdf" href="/blog/master-plan.pdf">
            <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M6 1v7M3 5.5 6 8.5 9 5.5M2 11h8" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" /></svg>
            PDF
          </a>
        </div>
        <nav className="home-small" aria-label="More">
          <a href={mail("kyte waitlist")}>kyte waitlist</a>
          <a href={mail("Reserve one of the first ten lymbs")}>Reserve a lymb</a>
          <a href={mail("Investing in Municipal Robotics")}>Investors</a>
        </nav>
      </PostCover>
    </div>
  );
}
