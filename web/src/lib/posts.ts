import { readFileSync } from "node:fs";
import { join } from "node:path";
import postsJson from "../../posts.json";

export interface Post {
  slug: string;
  title: string;
  subtitle: string;
  date: string;
  author: string;
  description: string;
  /** cover art id (see components/blog/covers); absent = the solar system */
  cover?: string;
}

export const posts: Post[] = [...postsJson].sort((a, b) => b.date.localeCompare(a.date));

export function getPost(slug: string): Post | undefined {
  return posts.find((p) => p.slug === slug);
}

/** The post body, compiled from Typst by `npm run posts`. Read at build time. */
export function getPostHtml(slug: string): string {
  return readFileSync(join(process.cwd(), "src/content/posts", `${slug}.html`), "utf8");
}

export function formatDate(date: string): string {
  return new Date(`${date}T12:00:00Z`).toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" });
}
