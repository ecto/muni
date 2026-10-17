"use client";

import { useState } from "react";
import { element, family, layout, links, maturityLabel } from "@/lib/plan";

// The periodic table of Municipal Robotics, as in the paper: solid cards are in use today,
// outlined ones are designed, dashed ones are concepts. Hover or tap a card to read it.
export function PeriodicTable() {
  const [num, setNum] = useState(1);
  const sel = element(num);
  const selFam = family(sel.family);
  const link = links[sel.name.toLowerCase()];

  return (
    <div className="ptable">
      <div className="ptable-grid">
        {layout.map(([key, cols]) => {
          const f = family(key);
          return (
            <div key={key} className="ptable-family" style={{ "--fam": f.color } as React.CSSProperties}>
              <div className="ptable-fam-name">{f.name}</div>
              <div className="ptable-cols">
                {cols.map((col, ci) => (
                  <div key={ci} className="ptable-col">
                    {col.map((n) => {
                      const e = element(n);
                      return (
                        <button
                          key={n}
                          type="button"
                          className={`pcard is-${e.maturity}${n === num ? " is-selected" : ""}`}
                          onMouseEnter={() => setNum(n)}
                          onFocus={() => setNum(n)}
                          onClick={() => setNum(n)}
                          aria-pressed={n === num}
                        >
                          <span className="pcard-num">{e.num}</span>
                          <span className="pcard-stage">{e.stage}</span>
                          <span className="pcard-name">{e.name.toLowerCase()}</span>
                          <span className="pcard-note">{e.note}</span>
                        </button>
                      );
                    })}
                  </div>
                ))}
              </div>
            </div>
          );
        })}
      </div>

      <div className="ptable-detail" style={{ "--fam": selFam.color } as React.CSSProperties} aria-live="polite">
        <span className="ptable-detail-name">{sel.name}</span>
        <span className="ptable-detail-meta">
          {selFam.name} · {maturityLabel[sel.maturity]} · first needed at {sel.stage[0].toUpperCase() + sel.stage.slice(1)}
        </span>
        <span className="ptable-detail-note">{sel.note}</span>
        {link && (
          <a className="ptable-detail-link" href={link} target="_blank" rel="noopener noreferrer">
            {link.replace(/^https:\/\//, "")} →
          </a>
        )}
      </div>

      <div className="ptable-key">
        <span><i className="key-swatch is-exists" /> in use today</span>
        <span><i className="key-swatch is-designed" /> designed</span>
        <span><i className="key-swatch is-concept" /> concept</span>
      </div>
    </div>
  );
}
