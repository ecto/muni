import type { Metadata } from "next";

export const metadata: Metadata = {
  title: "CAD Viewer",
  description:
    "Interactive 3D CAD viewer for BVR autonomous sidewalk rover models. Explore components and specifications.",
  alternates: {
    canonical: "https://muni.works/viewer",
  },
};

export default function ViewerLayout({ children }: { children: React.ReactNode }) {
  return children;
}
