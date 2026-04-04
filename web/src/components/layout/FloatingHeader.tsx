"use client";

import { useEffect, useState } from "react";
import Link from "next/link";
import { MuniLogo } from "./Header";
import { ArrowRight } from "@phosphor-icons/react";

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
        <Link href="/" className="site-nav-brand">
          <MuniLogo className="site-nav-logo" />
          <span className="site-nav-name">Municipal Robotics</span>
        </Link>

        <nav className="site-nav-links" aria-label="Main navigation">
          <Link href="/rover" className="site-nav-link">
            Rover
          </Link>
          <a href="/docs/whitepaper.pdf" className="site-nav-link" target="_blank" rel="noopener noreferrer">
            Whitepaper
          </a>
          <a href="https://github.com/ecto/muni" className="site-nav-link" target="_blank" rel="noopener noreferrer">
            GitHub
          </a>
        </nav>

        <a
          href="https://muni.cal.com/cam/30min"
          className="site-nav-cta"
        >
          Talk to Us
          <ArrowRight size={14} weight="bold" />
        </a>
      </div>
    </header>
  );
}
