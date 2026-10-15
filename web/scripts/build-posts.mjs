// Build blog posts from Typst sources: one .typ file → a web post and a PDF.
//
//   npm run posts
//
// Needs typst ≥ 0.14 on PATH. Vercel's build image has no typst, so the outputs
// are committed: src/content/posts/<slug>.html and public/blog/<slug>.pdf.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync, copyFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const web = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const posts = JSON.parse(readFileSync(join(web, "posts.json"), "utf8"));
const contentDir = join(web, "src/content/posts");
const pdfDir = join(web, "public/blog");
mkdirSync(contentDir, { recursive: true });
mkdirSync(pdfDir, { recursive: true });

for (const post of posts) {
  const root = resolve(web, post.root);
  const src = join(root, post.source);
  const common = ["--root", root, "--font-path", join(root, "fonts"), "--ignore-system-fonts"];

  // Web: the same source, built with target=html
  const htmlOut = join(contentDir, `${post.slug}.full.html`);
  execFileSync("typst", ["compile", ...common, "--features", "html", "--format", "html", "--input", "target=html", src, htmlOut], { stdio: "inherit" });
  const full = readFileSync(htmlOut, "utf8");
  const body = full.slice(full.indexOf("<body>") + 6, full.lastIndexOf("</body>")).trim();
  writeFileSync(join(contentDir, `${post.slug}.html`), body);
  execFileSync("rm", [htmlOut]);

  // Print: the PDF, linked from the post
  const pdfOut = join(pdfDir, `${post.slug}.pdf`);
  execFileSync("typst", ["compile", ...common, src, pdfOut], { stdio: "inherit" });

  console.log(`built ${post.slug}: ${(body.length / 1e6).toFixed(1)} MB html, ${pdfOut}`);
}
