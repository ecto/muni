import Image from "next/image";
import Link from "next/link";
import { Footer, FloatingHeader } from "@/components/layout";
import { PostCover } from "@/components/blog/PostCover";
import { StageStrip } from "@/components/home/StageStrip";
import { PeriodicTable } from "@/components/home/PeriodicTable";
import "@/styles/post.css";
import "@/styles/home.css";

// Product first, like a store: what you can use, what you can reserve, what is coming.
// The plan, for investors, follows the products.
const mail = (subject: string) => `mailto:info@muni.works?subject=${encodeURIComponent(subject)}`;

export default function HomePage() {
  return (
    <div className="post-page home-page">
      <FloatingHeader />

      <main>
        {/* frog: free, today */}
        <section className="tile tile-hero">
          <div className="tile-copy">
            <h2 className="tile-name is-mind">frog</h2>
            <p className="tile-line">A coding agent that runs on your Mac.</p>
            <p className="tile-sub">Free. Local models first, on our own inference engine.</p>
            <div className="tile-links">
              <a className="home-btn is-primary" href={mail("frog early access")}>Get early access</a>
            </div>
          </div>
          <div className="term" aria-label="An illustrative frog session">
            <div className="term-bar"><i /><i /><i /><span>frog · tang/Qwen3-30B (local)</span></div>
            <pre className="term-body">
              <span className="t-dim">› </span>fix the failing test in parser.rs{"\n"}
              <span className="t-tool">Read</span> src/parser.rs{"\n"}
              <span className="t-tool">Bash</span> cargo test parser{"\n"}
              <span className="t-dim">  1 failed: trailing comma in arrays</span>{"\n"}
              <span className="t-tool">Edit</span> src/parser.rs  <span className="t-ok">+4 −1</span>{"\n"}
              <span className="t-tool">Bash</span> cargo test{"\n"}
              <span className="t-ok">  ok. 212 passed</span>{"\n"}
              <span className="t-dim">done locally · 0 tokens sent anywhere</span>
            </pre>
          </div>
        </section>

        {/* the gateway: what we sell */}
        <section className="tile tile-gateway">
          <div className="tile-copy is-center">
            <h2 className="tile-name is-make">tang gateway</h2>
            <p className="tile-line">When local isn&apos;t enough.</p>
            <p className="tile-sub">frog&apos;s default cloud model. Your session stays warm, so you pay per finished task, not per token.</p>
            <div className="tile-links">
              <a className="home-btn is-primary" href={mail("tang gateway waitlist")}>Join the waitlist</a>
            </div>
          </div>
          <div className="handoff" aria-hidden="true">
            <span className="handoff-node">your Mac</span>
            <span className="handoff-line" />
            <span className="handoff-node is-lit">tang gateway</span>
          </div>
        </section>

        {/* the limb: reserve */}
        <section className="tile tile-limb">
          <div className="tile-copy is-center">
            <h2 className="tile-name is-body">limb</h2>
            <p className="tile-line">An arm that ships with its gap.</p>
            <p className="tile-sub">Six axes. Every unit measured against its own model, and the error printed on its receipt. $5,000.</p>
            <div className="tile-links">
              <a className="home-btn is-primary" href={mail("Reserve one of the first ten limbs")}>Reserve one of the first ten</a>
              <Link className="tile-more" href="/blog/master-plan">Learn more</Link>
            </div>
          </div>
          <Image className="tile-img" src="/images/home/limb.png" alt="The limb, Rose fork v9: a six-axis arm" width={927} height={702} priority />
        </section>

        {/* what comes next */}
        <div className="tile-pair">
          <section className="tile tile-half">
            <div className="tile-copy is-center">
              <h2 className="tile-name is-body">pair</h2>
              <p className="tile-line">Two arms. Any table.</p>
              <p className="tile-sub">Folds laundry, tends a machine, builds the next limb. $12,000, target.</p>
              <span className="tile-soon">Coming later</span>
            </div>
            <Image className="tile-img" src="/images/home/pair.png" alt="A pair of limbs at a bench" width={743} height={876} />
          </section>
          <section className="tile tile-half">
            <div className="tile-copy is-center">
              <h2 className="tile-name is-body">Rose</h2>
              <p className="tile-line">A pair on legs.</p>
              <p className="tile-sub">Our humanoid, in three sizes: mini, full and mega.</p>
              <span className="tile-soon">Coming later</span>
            </div>
            <Image className="tile-img is-tall" src="/images/home/rose.png" alt="Rose, the humanoid" width={437} height={1400} />
          </section>
        </div>

        {/* the plan, for investors */}
        <PostCover title="Robots that build robots" subtitle="Our plan, from a desk to a star. Software pays for the arm; the arm builds the rest.">
          <div className="post-byline home-cta">
            <Link className="home-btn is-primary" href="/blog/master-plan">Read the plan</Link>
            <a className="pdf" href="/blog/master-plan.pdf">
              <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M6 1v7M3 5.5 6 8.5 9 5.5M2 11h8" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" /></svg>
              PDF
            </a>
            <a className="tile-more" href={mail("Investing in Municipal Robotics")}>Talk to us about investing</a>
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
            <p>Every product above sits on a stack we own, from the language parts are written in to the physics they are tested in.</p>
          </div>
          <PeriodicTable />
        </section>
      </main>

      <div className="landing-footer">
        <div className="landing-container">
          <Footer />
        </div>
      </div>
    </div>
  );
}
