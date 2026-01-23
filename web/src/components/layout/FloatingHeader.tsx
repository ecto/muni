"use client";

import Link from "next/link";
import { MuniLogo } from "./Header";
import { ArrowRight } from "@phosphor-icons/react";

export function FloatingHeader() {
  return (
    <header className="floating-header">
      <div className="floating-header-content">
        <Link href="/" className="floating-header-brand">
          <MuniLogo className="floating-header-logo" />
        </Link>

        <nav className="floating-header-nav" aria-label="Main navigation">
          <Link href="/rover" className="floating-header-link">
            Rover
          </Link>
          <a href="/docs/whitepaper.pdf" className="floating-header-link" target="_blank" rel="noopener noreferrer">
            Whitepaper
          </a>
          <a href="https://github.com/ecto/muni" className="floating-header-link" target="_blank" rel="noopener noreferrer">
            GitHub
          </a>
          <a
            href="https://muni.cal.com/cam/30min"
            className="floating-header-cta"
          >
            Talk to Us
            <ArrowRight size={14} weight="bold" />
          </a>
        </nav>
      </div>
    </header>
  );
}
