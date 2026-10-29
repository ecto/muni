import { coverFor } from "./covers";

// The shared cover shell: full-bleed art behind the title, subtitle, byline and
// the scroll cue. Each post picks its art by `cover` in posts.json; the default
// is the solar system.
export function PostCover({ title, subtitle, children, scrollTo, cover }: { title: string; subtitle: string; children?: React.ReactNode; scrollTo?: string; cover?: string }) {
  const { Art } = coverFor(cover);
  return (
    <header className="post-cover">
      <Art />
      <div className="post-cover-text">
        <h1>{title}</h1>
        <p className="post-subtitle">{subtitle}</p>
        {children}
      </div>
      {scrollTo && (
        <a className="post-cover-scroll" href={`#${scrollTo}`} aria-label="Scroll to the text">
          <span>Scroll</span>
          <svg viewBox="0 0 12 12" aria-hidden="true"><path d="M2.5 4.5 6 8l3.5-3.5" fill="none" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" /></svg>
        </a>
      )}
    </header>
  );
}
