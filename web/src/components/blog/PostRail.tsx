"use client";

import { useEffect, useRef, useState } from "react";

// A scrollbar with a map in it. The page's own sections become ticks on a rail
// that runs from Earth (the top) to the Sun (the end); the reader is the probe
// from the cover, travelling down it. Desktop with a fine pointer only; phones
// keep their native scrollbar.

interface Mark {
  id: string;
  top: number; // 0..1 along the document
  label: string;
  num: string;
  kind: "part" | "section";
}

function readMarks(): Mark[] {
  const doc = document.documentElement;
  const max = Math.max(1, doc.scrollHeight - window.innerHeight);
  const marks: Mark[] = [];
  document.querySelectorAll<HTMLElement>(".post section.part, .post > h2, .post .back-matter > h2").forEach((el, i) => {
    const isPart = el.matches("section.part");
    const heading = isPart ? el.querySelector("h2") : el;
    if (!heading) return;
    if (!el.id) el.id = isPart ? `part-${i}` : `sec-${i}`;
    const num = isPart ? (el.querySelector(".part-label")?.textContent ?? "").replace(/^Part\s*/i, "") : (heading.querySelector(".sec-num")?.textContent ?? "");
    const label = (heading.textContent ?? "").replace(num, "").trim();
    const y = el.getBoundingClientRect().top + window.scrollY;
    marks.push({ id: el.id, top: Math.min(1, Math.max(0, (y - 80) / max)), label, num, kind: isPart ? "part" : "section" });
  });
  return marks;
}

export function PostRail() {
  const [on, setOn] = useState(false);
  const [marks, setMarks] = useState<Mark[]>([]);
  const [progress, setProgress] = useState(0);
  const [current, setCurrent] = useState<Mark | null>(null);
  const [awake, setAwake] = useState(false);
  const sleep = useRef<ReturnType<typeof setTimeout> | null>(null);
  const railRef = useRef<HTMLDivElement>(null);
  const dragging = useRef(false);

  useEffect(() => {
    const mq = window.matchMedia("(pointer: fine) and (min-width: 60rem)");
    const apply = () => {
      setOn(mq.matches);
      document.documentElement.classList.toggle("post-rail-on", mq.matches);
    };
    apply();
    mq.addEventListener("change", apply);
    return () => {
      mq.removeEventListener("change", apply);
      document.documentElement.classList.remove("post-rail-on");
    };
  }, []);

  useEffect(() => {
    if (!on) return;
    const measure = () => setMarks(readMarks());
    const onScroll = () => {
      const doc = document.documentElement;
      const p = Math.min(1, Math.max(0, window.scrollY / Math.max(1, doc.scrollHeight - window.innerHeight)));
      setProgress(p);
      setAwake(true);
      if (sleep.current) clearTimeout(sleep.current);
      sleep.current = setTimeout(() => setAwake(false), 1400);
    };
    measure();
    onScroll();
    setAwake(false);
    const ro = new ResizeObserver(measure);
    ro.observe(document.body);
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => {
      ro.disconnect();
      window.removeEventListener("scroll", onScroll);
      if (sleep.current) clearTimeout(sleep.current);
    };
  }, [on]);

  useEffect(() => {
    let c: Mark | null = null;
    for (const m of marks) if (m.kind === "section" && m.top <= progress + 0.002) c = m;
    setCurrent(c);
  }, [marks, progress]);

  // drag the probe, or click anywhere on the rail, to travel
  useEffect(() => {
    if (!on) return;
    const rail = railRef.current;
    if (!rail) return;
    const go = (clientY: number) => {
      const r = rail.getBoundingClientRect();
      const p = Math.min(1, Math.max(0, (clientY - r.top) / r.height));
      const doc = document.documentElement;
      window.scrollTo({ top: p * (doc.scrollHeight - window.innerHeight) });
    };
    const down = (e: PointerEvent) => {
      if ((e.target as HTMLElement).closest("a")) return;
      dragging.current = true;
      rail.setPointerCapture(e.pointerId);
      go(e.clientY);
    };
    const move = (e: PointerEvent) => dragging.current && go(e.clientY);
    const up = () => (dragging.current = false);
    rail.addEventListener("pointerdown", down);
    rail.addEventListener("pointermove", move);
    rail.addEventListener("pointerup", up);
    return () => {
      rail.removeEventListener("pointerdown", down);
      rail.removeEventListener("pointermove", move);
      rail.removeEventListener("pointerup", up);
    };
  }, [on]);

  if (!on) return null;

  return (
    <nav className={`post-rail${awake ? " is-awake" : ""}`} aria-label="Sections">
      <span className="post-rail-end post-rail-earth">Earth</span>
      <div className="post-rail-track" ref={railRef}>
        <div className="post-rail-trail" style={{ height: `${progress * 100}%` }} />
        {marks.map((m) => (
          <a
            key={m.id}
            href={`#${m.id}`}
            className={`post-rail-mark is-${m.kind}${current?.id === m.id ? " is-current" : ""}`}
            style={{ top: `${m.top * 100}%` }}
            aria-label={m.kind === "part" ? `Part ${m.num}: ${m.label}` : `${m.num} ${m.label}`}
          >
            <span className="post-rail-tip">
              {m.num && <b>{m.kind === "part" ? `Part ${m.num}` : m.num}</b>}
              {m.label}
            </span>
          </a>
        ))}
        <div className="post-rail-probe" style={{ top: `${progress * 100}%` }}>
          {current && (
            <span className="post-rail-now">
              <b>{current.num}</b>
              {current.label}
            </span>
          )}
        </div>
      </div>
      <span className="post-rail-end post-rail-sun">Sun</span>
    </nav>
  );
}
