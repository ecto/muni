"use client";

import { useEffect, useRef } from "react";

interface ScrollRevealOptions {
  threshold?: number;
  rootMargin?: string;
}

export function useScrollReveal<T extends HTMLElement>(
  options: ScrollRevealOptions = {}
) {
  const ref = useRef<T>(null);

  useEffect(() => {
    const el = ref.current;
    if (!el) return;

    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (prefersReducedMotion) {
      el.classList.add("revealed");
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          el.classList.add("revealed");
          observer.unobserve(el);
        }
      },
      {
        threshold: options.threshold ?? 0.15,
        rootMargin: options.rootMargin ?? "0px 0px -40px 0px",
      }
    );

    observer.observe(el);
    return () => observer.disconnect();
  }, [options.threshold, options.rootMargin]);

  return ref;
}

export function useScrollRevealGroup(
  containerSelector: string,
  childSelector: string,
  staggerMs = 80
) {
  useEffect(() => {
    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    const containers = document.querySelectorAll(containerSelector);

    if (prefersReducedMotion) {
      containers.forEach((container) => {
        container.querySelectorAll(childSelector).forEach((child) => {
          (child as HTMLElement).classList.add("revealed");
        });
      });
      return;
    }

    const observers: IntersectionObserver[] = [];

    containers.forEach((container) => {
      const observer = new IntersectionObserver(
        ([entry]) => {
          if (entry.isIntersecting) {
            const children = container.querySelectorAll(childSelector);
            children.forEach((child, i) => {
              setTimeout(() => {
                (child as HTMLElement).classList.add("revealed");
              }, i * staggerMs);
            });
            observer.unobserve(container);
          }
        },
        { threshold: 0.1, rootMargin: "0px 0px -40px 0px" }
      );

      observer.observe(container);
      observers.push(observer);
    });

    return () => observers.forEach((o) => o.disconnect());
  }, [containerSelector, childSelector, staggerMs]);
}
