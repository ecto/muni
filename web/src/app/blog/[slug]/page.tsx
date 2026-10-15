import type { Metadata } from "next";
import { notFound } from "next/navigation";
import { Footer, FloatingHeader } from "@/components/layout";
import { PostCover } from "@/components/blog/PostCover";
import { PostMotion } from "@/components/blog/PostMotion";
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
        <div className="post-meta">
          <span>{post.author}</span>
          <span aria-hidden="true">·</span>
          <time dateTime={post.date}>{formatDate(post.date)}</time>
          <span aria-hidden="true">·</span>
          <a href={`/blog/${post.slug}.pdf`}>PDF</a>
        </div>
      </PostCover>
      {/* Compiled from the Typst source by `npm run posts`; the same file builds the PDF. */}
      <article className="post" dangerouslySetInnerHTML={{ __html: html }} />
      <PostMotion />
      <div className="landing-footer">
        <div className="landing-container">
          <Footer />
        </div>
      </div>
    </div>
  );
}
