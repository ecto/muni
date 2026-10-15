import type { Metadata } from "next";
import Link from "next/link";
import { Footer, FloatingHeader } from "@/components/layout";
import { formatDate, posts } from "@/lib/posts";
import "@/styles/post.css";

export const metadata: Metadata = {
  title: "Blog",
  description: "Research and plans from Municipal Robotics, written in Typst and published as both web pages and PDFs.",
  alternates: { canonical: "https://muni.works/blog" },
};

export default function BlogIndex() {
  return (
    <div className="post-page">
      <FloatingHeader />
      <main className="blog-index">
        <h1>Blog</h1>
        <p className="blog-index-lede">Research and plans, each one a web page and a PDF from the same source.</p>
        <ol className="blog-list">
          {posts.map((p) => (
            <li key={p.slug}>
              <Link href={`/blog/${p.slug}`}>
                <time dateTime={p.date}>{formatDate(p.date)}</time>
                <span className="blog-list-title">{p.title}</span>
                <span className="blog-list-desc">{p.description}</span>
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
