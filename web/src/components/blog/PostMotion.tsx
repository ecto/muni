"use client";

import { useEffect } from "react";

// Figures are complete at rest. As each drawn figure scrolls into view it wipes
// in left to right, the way a plotter would draw it. Skipped for reduced motion.
export function PostMotion() {
  useEffect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    const targets = Array.from(document.querySelectorAll<HTMLElement>(".post figure > svg, .post .part-art > svg"));
    const below = targets.filter((el) => el.getBoundingClientRect().top > window.innerHeight);
    below.forEach((el) => el.classList.add("draw-pending"));
    // watch each figure's container: a clipped element never reports itself as visible
    const io = new IntersectionObserver(
      (entries) => {
        for (const e of entries) {
          if (!e.isIntersecting) continue;
          const svg = e.target.querySelector(":scope > svg");
          svg?.classList.add("draw-in");
          svg?.classList.remove("draw-pending");
          io.unobserve(e.target);
        }
      },
      { rootMargin: "0px 0px -10% 0px" },
    );
    below.forEach((el) => el.parentElement && io.observe(el.parentElement));
    return () => io.disconnect();
  }, []);
  return null;
}
