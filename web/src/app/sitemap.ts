import type { MetadataRoute } from "next";
import { states } from "@/lib/data/states";

export default function sitemap(): MetadataRoute.Sitemap {
  const baseUrl = "https://muni.works";

  const staticPages: MetadataRoute.Sitemap = [
    { url: baseUrl, lastModified: new Date(), priority: 1.0 },
    { url: `${baseUrl}/rover`, lastModified: new Date(), priority: 0.9 },
    { url: `${baseUrl}/about`, lastModified: new Date(), priority: 0.7 },
    { url: `${baseUrl}/docs`, lastModified: new Date(), priority: 0.6 },
    { url: `${baseUrl}/investors`, lastModified: new Date(), priority: 0.5 },
    { url: `${baseUrl}/log`, lastModified: new Date(), priority: 0.5 },
    { url: `${baseUrl}/viewer`, lastModified: new Date(), priority: 0.6 },
    { url: `${baseUrl}/sidewalk-laws`, lastModified: new Date(), priority: 0.8 },
  ];

  const statePages: MetadataRoute.Sitemap = states.map((state) => ({
    url: `${baseUrl}/sidewalk-laws/${state.slug}`,
    lastModified: new Date(),
    priority: 0.8,
  }));

  return [...staticPages, ...statePages];
}
