"use client";

import { useEffect, useRef, useState } from "react";

const SEQUENCES = [
  [
    { text: "$ muni deploy rover frog-0 --all", typing: true },
    { text: "  ✓ firmware uploaded (bvrd v2.4.1)", delay: 600 },
    { text: "  ✓ policy loaded (bc-teleop-v0.1.0)", delay: 400 },
    { text: "  ✓ safety zones: active", delay: 300 },
    { text: "  ✓ fleet connected: 3/3 rovers online", delay: 500 },
    { text: "", delay: 600 },
    { text: "frog-0 > status", typing: true },
    { text: "  mode:     autonomous", delay: 200 },
    { text: "  battery:  94%", delay: 150 },
    { text: "  cleared:  12.4 mi", delay: 150 },
    { text: "  uptime:   6h 23m", delay: 150 },
  ],
  [
    { text: "$ muni rover scan", typing: true },
    { text: "  scanning CAN bus...", delay: 800 },
    { text: "  VESC 1 (FL)  48V  32°C  ✓", delay: 300 },
    { text: "  VESC 2 (FR)  48V  31°C  ✓", delay: 200 },
    { text: "  VESC 3 (RL)  48V  33°C  ✓", delay: 200 },
    { text: "  VESC 4 (RR)  48V  31°C  ✓", delay: 200 },
    { text: "  MCU   0x0B00  LED OK  ✓", delay: 300 },
    { text: "", delay: 400 },
    { text: "  all systems nominal", delay: 200 },
  ],
];

interface Line {
  text: string;
  typing?: boolean;
  delay?: number;
}

export function HeroTerminal() {
  const [lines, setLines] = useState<string[]>([]);
  const [currentText, setCurrentText] = useState("");
  const [showCursor, setShowCursor] = useState(true);
  const seqRef = useRef(0);
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const prefersReducedMotion = window.matchMedia(
      "(prefers-reduced-motion: reduce)"
    ).matches;

    if (prefersReducedMotion) {
      setLines(SEQUENCES[0].map((l) => l.text));
      setCurrentText("");
      return;
    }

    let cancelled = false;

    async function sleep(ms: number) {
      return new Promise((r) => setTimeout(r, ms));
    }

    async function typeLine(text: string) {
      for (let i = 0; i <= text.length; i++) {
        if (cancelled) return;
        setCurrentText(text.slice(0, i));
        await sleep(25 + Math.random() * 20);
      }
    }

    async function runSequence(seq: Line[]) {
      setLines([]);
      setCurrentText("");

      for (const line of seq) {
        if (cancelled) return;

        if (line.typing) {
          await typeLine(line.text);
          await sleep(300);
          setLines((prev) => [...prev, line.text]);
          setCurrentText("");
        } else {
          await sleep(line.delay ?? 200);
          setLines((prev) => [...prev, line.text]);
        }
      }
    }

    async function loop() {
      while (!cancelled) {
        const seq = SEQUENCES[seqRef.current % SEQUENCES.length];
        await runSequence(seq);
        await sleep(4000);
        seqRef.current++;
      }
    }

    loop();

    return () => {
      cancelled = true;
    };
  }, []);

  // Cursor blink
  useEffect(() => {
    const interval = setInterval(() => setShowCursor((c) => !c), 530);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="hero-terminal" ref={containerRef}>
      <div className="hero-terminal-chrome">
        <span className="hero-terminal-dot" />
        <span className="hero-terminal-dot" />
        <span className="hero-terminal-dot" />
      </div>
      <div className="hero-terminal-body">
        {lines.map((line, i) => (
          <div key={i} className={`hero-terminal-line${line.startsWith("  ✓") ? " hero-terminal-success" : ""}`}>
            {line}
          </div>
        ))}
        {currentText !== undefined && (
          <div className="hero-terminal-line hero-terminal-active">
            {currentText}
            <span className={`hero-terminal-cursor ${showCursor ? "" : "hero-terminal-cursor-hidden"}`}>█</span>
          </div>
        )}
      </div>
    </div>
  );
}
