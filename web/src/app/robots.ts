import type { MetadataRoute } from "next";

export default function robots(): MetadataRoute.Robots {
  return {
    rules: [{ userAgent: "*", allow: "/", disallow: ["/success", "/cancel"] }],
    sitemap: "https://muni.works/sitemap.xml",
  };
}
