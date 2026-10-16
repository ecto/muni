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

// A <label>'s metadata value from the source, or null if it publishes none.
function queryMetadata(common, src, label) {
  try {
    const out = execFileSync("typst", ["query", ...common, src, label, "--field", "value", "--one"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] });
    return JSON.parse(out);
  } catch {
    return null;
  }
}

// Product names, as [name, family] pairs, if the source publishes them as <brand-names> metadata.
function brandNames(common, src) {
  return queryMetadata(common, src, "<brand-names>") ?? [];
}

// Wrap each product name in <span class="name n-<family>">, in text only: never inside
// tags, SVG frames (whose text is drawn), code or the head of a link's attributes.
function colorNames(html, names) {
  if (!names.length) return html;
  const family = new Map(names);
  const re = new RegExp(`\\b(${[...family.keys()].sort((a, b) => b.length - a.length).map((n) => n.replace(/[-]/g, "\\$&")).join("|")})\\b`, "gi");
  let skip = 0;
  return html
    .split(/(<[^>]+>)/)
    .map((part) => {
      if (part.startsWith("<")) {
        const m = part.match(/^<(\/?)(svg|code|pre|script|style)\b/);
        if (m) skip += m[1] ? -1 : part.endsWith("/>") ? 0 : 1;
        return part;
      }
      return skip > 0 ? part : part.replace(re, (n) => `<span class="name n-${family.get(n.toLowerCase())}">${n}</span>`);
    })
    .join("");
}

for (const post of posts) {
  const root = resolve(web, post.root);
  const src = join(root, post.source);
  const common = ["--root", root, "--font-path", join(root, "fonts"), "--ignore-system-fonts"];

  // Web: the same source, built with target=html
  const htmlOut = join(contentDir, `${post.slug}.full.html`);
  execFileSync("typst", ["compile", ...common, "--features", "html", "--format", "html", "--input", "target=html", src, htmlOut], { stdio: "inherit" });
  const full = readFileSync(htmlOut, "utf8");
  let body = full.slice(full.indexOf("<body>") + 6, full.lastIndexOf("</body>")).trim();
  body = colorNames(body, brandNames(common, src));
  const table = queryMetadata(common, src, "<elements-data>");
  if (table) writeFileSync(join(contentDir, "..", "elements.json"), JSON.stringify(table, null, 2) + "\n");
  writeFileSync(join(contentDir, `${post.slug}.html`), body);
  execFileSync("rm", [htmlOut]);

  // Print: the PDF, linked from the post
  const pdfOut = join(pdfDir, `${post.slug}.pdf`);
  execFileSync("typst", ["compile", ...common, src, pdfOut], { stdio: "inherit" });

  console.log(`built ${post.slug}: ${(body.length / 1e6).toFixed(1)} MB html, ${pdfOut}`);
}
