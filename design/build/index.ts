/**
 * Build runner — generates all platform-specific token files.
 *
 * Usage: npx tsx build/index.ts
 */

import { writeFileSync, mkdirSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { buildCSS } from "./css";
import { buildTailwind } from "./tailwind";
import { buildTypst } from "./typst";

const __dirname = dirname(fileURLToPath(import.meta.url));
const distDir = join(__dirname, "..", "dist");

mkdirSync(distDir, { recursive: true });

const outputs = [
  { name: "tokens.css", build: buildCSS },
  { name: "tailwind-theme.css", build: buildTailwind },
  { name: "tokens.typ", build: buildTypst },
];

for (const { name, build } of outputs) {
  const content = build();
  const path = join(distDir, name);
  writeFileSync(path, content, "utf-8");
  const bytes = Buffer.byteLength(content);
  console.log(`  ${name} (${bytes} bytes)`);
}

console.log(`\nDone — ${outputs.length} files written to dist/`);
