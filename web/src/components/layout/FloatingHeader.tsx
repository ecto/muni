import Link from "next/link";
import { MuniLogo } from "./Header";

export function FloatingHeader() {
  return (
    <header className="floating-header">
      <div className="floating-header-content">
        <Link href="/" className="floating-header-brand">
          <MuniLogo className="floating-header-logo" />
          <span className="floating-header-name">Municipal Robotics</span>
        </Link>

        <nav className="floating-header-nav" aria-label="Main navigation">
          <a href="/docs/whitepaper.pdf" className="floating-header-link" target="_blank" rel="noopener noreferrer">
            Whitepaper<span className="sr-only"> (opens in new tab)</span>
          </a>
          <a href="https://github.com/ecto/muni" className="floating-header-link" target="_blank" rel="noopener noreferrer">
            GitHub<span className="sr-only"> (opens in new tab)</span>
          </a>
          <a
            href="https://buy.stripe.com/dRm8wH3aL91u5mybf3grS00"
            className="floating-header-cta"
          >
            Reserve Now
          </a>
        </nav>
      </div>
    </header>
  );
}
