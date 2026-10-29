import type { Metadata } from "next";
import Link from "next/link";
import { Footer, FloatingHeader } from "@/components/layout";
import { formatDate, posts } from "@/lib/posts";
import "@/styles/post.css";

export const metadata: Metadata = {
  title: "Research",
  description: "Research and plans from Municipal Robotics, written in Typst and published as both web pages and PDFs.",
  alternates: { canonical: "https://muni.works/research" },
  openGraph: {
    title: "Research | Municipal Robotics",
    description: "Research and plans from Municipal Robotics, written in Typst and published as both web pages and PDFs.",
    type: "website",
    url: "/research",
    siteName: "Municipal Robotics",
    images: [{ url: "/og/research.png", width: 1200, height: 630, alt: "Municipal Robotics research" }],
  },
  twitter: { card: "summary_large_image", images: ["/og/research.png"] },
};

export default function ResearchIndex() {
  return (
    <div className="post-page">
      <FloatingHeader />
      <main className="blog-index">
        <h1>Research</h1>
        <p className="blog-index-lede">Each one a web page and a PDF, from the same source.</p>
        <ol className="blog-list">
          {posts.map((p, i) => (
            <li key={p.slug}>
              <Link href={`/research/${p.slug}`}>
                <span className="blog-list-num">{String(posts.length - i).padStart(2, "0")}</span>
                <span className="blog-list-body">
                  <span className="blog-list-title">{p.title}</span>
                  <span className="blog-list-desc">{p.description}</span>
                </span>
                <time dateTime={p.date}>{formatDate(p.date)}</time>
              </Link>
            </li>
          ))}
        </ol>
      </main>
      <div className="landing-footer">
        <div className="landing-container">
          <Footer />
        </div>
      </div>
    </div>
  );
}
