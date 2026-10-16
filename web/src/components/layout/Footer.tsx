import Link from "next/link";
import { MuniLogo } from "./Header";

export function Footer() {
  const year = new Date().getFullYear();

  return (
    <footer className="site-footer">
      <div className="site-footer-top">
        <Link href="/" className="site-footer-brand" aria-label="Home">
          <MuniLogo className="site-footer-logo" />
        </Link>

        <nav className="site-footer-links">
          <Link href="/blog/master-plan">Plan</Link>
          <Link href="/about">About</Link>
          <Link href="/blog">Blog</Link>
          <a href="/blog/master-plan.pdf" target="_blank" rel="noopener noreferrer">PDF</a>
          <a href="https://github.com/ecto/muni" target="_blank" rel="noopener noreferrer">GitHub</a>
          <a href="mailto:info@muni.works">Contact</a>
        </nav>
      </div>

      <div className="site-footer-bottom">
        <span>&copy; {year} Municipal Robotics</span>
        <span className="site-footer-sep">&middot;</span>
        <span>Cleveland, Ohio</span>
      </div>
    </footer>
  );
}
