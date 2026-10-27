"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { MuniLogo } from "./Header";

export function FloatingHeader() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    function onScroll() {
      setScrolled(window.scrollY > 60);
    }
    window.addEventListener("scroll", onScroll, { passive: true });
    onScroll();
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header className={`site-nav ${scrolled ? "site-nav-scrolled" : ""}`}>
      <div className="site-nav-inner">
        <Link href="/" className="site-nav-brand" aria-label="Home">
          <MuniLogo className="site-nav-logo" />
        </Link>

        <nav className="site-nav-right" aria-label="Main navigation">
          <Link href="/research/vision" className="site-nav-link">Vision</Link>
          <Link href="/research" className="site-nav-link">Research</Link>
          <a href="https://github.com/ecto/muni" className="site-nav-link" target="_blank" rel="noopener noreferrer">GitHub</a>
        </nav>
      </div>
    </header>
  );
}
