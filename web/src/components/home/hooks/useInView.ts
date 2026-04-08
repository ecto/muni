"use client";

import { useRef, useState, useEffect } from "react";

export function useInView(margin = "200px"): [React.RefObject<HTMLDivElement | null>, boolean] {
  const ref = useRef<HTMLDivElement | null>(null);
  const [inView, setInView] = useState(false);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const observer = new IntersectionObserver(
      ([entry]) => setInView(entry.isIntersecting),
      { rootMargin: margin }
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, [margin]);

  return [ref, inView];
}
