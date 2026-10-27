// Small animated glyphs for the element cards, after the project cards on campedersen.com.
// Each draws in currentColor; the motion lives in home.css under the eg- prefix.

// frog writes software: one line at a time, the cursor riding the end of the line as it types,
// then the line lifts away as the next begins. Two lines take turns, so the feed never stops.
export const FrogGlyph = () => (
  <svg viewBox="0 0 64 32" fill="none" stroke="currentColor" strokeWidth="1.5" aria-hidden>
    <path d="M 4 17 L 8 20 L 4 23" strokeWidth="1.2" />
    <g className="eg-frog-line">
      <line x1="13" y1="20" x2="45" y2="20" style={{ "--len": 32 } as React.CSSProperties} />
      <rect x="13" y="16.5" width="2.5" height="7" fill="currentColor" stroke="none" style={{ "--len": 32 } as React.CSSProperties} />
    </g>
    <g className="eg-frog-line">
      <line x1="19" y1="20" x2="41" y2="20" style={{ "--len": 22 } as React.CSSProperties} />
      <rect x="19" y="16.5" width="2.5" height="7" fill="currentColor" stroke="none" style={{ "--len": 22 } as React.CSSProperties} />
    </g>
  </svg>
);

export const TangGlyph = () => (
  <svg viewBox="0 0 64 32" fill="none" stroke="currentColor" strokeWidth="1.4" aria-hidden>
    <line x1="2" y1="16" x2="62" y2="16" opacity="0.3" strokeDasharray="1 2" />
    <g className="eg-tang-wave">
      <path d="M -32 16 Q -24 6 -16 16 T 0 16 T 16 16 T 32 16 T 48 16 T 64 16 T 80 16 T 96 16" />
    </g>
  </svg>
);

export const VcadGlyph = () => (
  <svg viewBox="0 0 64 32" fill="none" stroke="currentColor" strokeWidth="1.1" aria-hidden>
    <g className="eg-vcad-box">
      <path d="M 10 14 L 30 14 L 40 9 L 20 9 Z" />
      <path d="M 10 14 L 30 14 L 30 24 L 10 24 Z" />
      <path d="M 30 14 L 40 9 L 40 19 L 30 24 Z" />
    </g>
    <g className="eg-vcad-dim" strokeWidth="0.9">
      <line x1="42" y1="9" x2="58" y2="9" opacity="0.55" />
      <line x1="42" y1="19" x2="58" y2="19" opacity="0.55" />
      <line x1="55" y1="11" x2="55" y2="17" />
    </g>
    <g className="eg-vcad-dim" fill="currentColor" stroke="none">
      <path d="M 55 9 L 53.4 11.5 L 56.6 11.5 Z" />
      <path d="M 55 19 L 53.4 16.5 L 56.6 16.5 Z" />
    </g>
  </svg>
);

export const PhyzGlyph = () => (
  <svg viewBox="0 0 64 32" fill="none" stroke="currentColor" strokeWidth="1.2" aria-hidden>
    <line x1="2" y1="28" x2="62" y2="28" />
    <g className="eg-phyz-shadow">
      <ellipse cx="32" cy="29" rx="4" ry="0.8" fill="currentColor" stroke="none" opacity="0.35" />
    </g>
    <g className="eg-phyz-ball">
      <circle cx="32" cy="8" r="3.5" fill="currentColor" stroke="none" />
    </g>
  </svg>
);

export const glyphs: Record<string, () => React.JSX.Element> = { frog: FrogGlyph, tang: TangGlyph, vcad: VcadGlyph, phyz: PhyzGlyph };
