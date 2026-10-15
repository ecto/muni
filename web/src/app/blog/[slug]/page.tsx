import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { Footer, FloatingHeader } from "@/components/layout";
import { PostCover } from "@/components/blog/PostCover";
import { formatDate, getPost, getPostHtml, posts } from "@/lib/posts";
import "@/styles/post.css";

export function generateStaticParams() {
  return posts.map((p) => ({ slug: p.slug }));
}

export function generateMetadata({ params }: { params: { slug: string } }): Metadata {
  const post = getPost(params.slug);
  if (!post) return {};
  return {
    title: post.title,
    description: post.description,
    alternates: { canonical: `https://muni.works/blog/${post.slug}` },
    openGraph: { title: post.title, description: post.description, type: "article", publishedTime: post.date },
  };
}

export default function PostPage({ params }: { params: { slug: string } }) {
  const post = getPost(params.slug);
  if (!post) notFound();
  const html = getPostHtml(post.slug);

  return (
    <div className="post-page">
      <FloatingHeader />
      <PostCover title={post.title} subtitle={post.subtitle}>
        <div className="post-byline">
          <span className="by">{post.author}</span>
          <time dateTime={post.date}>{formatDate(post.date)}</time>
          <a className="pdf" href={`/blog/${post.slug}.pdf`}>
            <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M6 1v7M3 5.5 6 8.5 9 5.5M2 11h8" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" /></svg>
            Read the PDF
          </a>
        </div>
      </PostCover>
      {/* Compiled from the Typst source by `npm run posts`; the same file builds the PDF. */}
      <article className="post" dangerouslySetInnerHTML={{ __html: html }} />
      <div className="landing-footer">
        <div className="landing-container">
          <Footer />
        </div>
      </div>
    </div>
  );
}
