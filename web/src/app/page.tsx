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
      <PostCover title="Robots that build robots" subtitle="An autonomous company, amplified by intelligence. It starts with frog, a free coding agent for your Mac.">
        <div className="post-byline home-cta">
          <a className="home-btn is-primary" href={mail("frog early access")}>Get frog</a>
          <Link className="home-btn is-quiet" href="/blog/master-plan">Read the plan</Link>
        </div>
      </PostCover>
    </div>
  );
}
