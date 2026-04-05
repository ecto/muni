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
        <nav className="site-nav-left" aria-label="Main navigation">
          <Link href="/rover" className="site-nav-link">Rover</Link>
          <Link href="/about" className="site-nav-link">About</Link>
        </nav>

        <Link href="/" className="site-nav-brand" aria-label="Home">
          <MuniLogo className="site-nav-logo" />
        </Link>

        <div className="site-nav-right">
          <a href="https://github.com/ecto/muni" className="site-nav-link" target="_blank" rel="noopener noreferrer">GitHub</a>
          <a href="https://muni.cal.com/cam/30min" className="site-nav-cta">Talk to Us</a>
        </div>
      </div>
    </header>
  );
}
