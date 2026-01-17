"use client";

import { useEffect, useState } from "react";

// TODO: Replace with real session replay data from Cleveland deployments
// Load actual .rrd session recordings, show real routes cleared with timestamps
// Format: "Recorded [date] - [miles] cleared in [duration]"
// This will be much more credible than the current visualization

export function CoverageMapViewer() {
  const [isClient, setIsClient] = useState(false);

  useEffect(() => {
    setIsClient(true);
  }, []);

  if (!isClient) {
    return <div className="coverage-map-viewer" />;
  }

  return (
    <div className="coverage-map-viewer" role="img" aria-label="Animated map showing three autonomous rovers clearing 50 miles of sidewalk paths">
      <svg
        viewBox="0 0 800 450"
        className="coverage-map-svg"
        style={{ width: "100%", height: "100%" }}
        aria-hidden="true"
      >
        {/* Background grid */}
        <defs>
          <pattern
            id="grid"
            width="40"
            height="40"
            patternUnits="userSpaceOnUse"
          >
            <path
              d="M 40 0 L 0 0 0 40"
              fill="none"
              stroke="currentColor"
              strokeWidth="0.5"
              opacity="0.1"
            />
          </pattern>

          {/* Masks for clearing effect - snow disappears as rovers pass */}
          <mask id="clearMask1">
            <rect width="800" height="450" fill="white" />
            <rect x="0" y="100" width="0" height="20" fill="black">
              <animate
                attributeName="width"
                from="0"
                to="800"
                dur="15s"
                repeatCount="indefinite"
              />
            </rect>
          </mask>

          <mask id="clearMask2">
            <rect width="800" height="450" fill="white" />
            <rect x="0" y="200" width="0" height="20" fill="black">
              <animate
                attributeName="width"
                from="0"
                to="800"
                dur="15s"
                begin="2s"
                repeatCount="indefinite"
              />
            </rect>
          </mask>

          <mask id="clearMask3">
            <rect width="800" height="450" fill="white" />
            <rect x="0" y="330" width="0" height="20" fill="black">
              <animate
                attributeName="width"
                from="0"
                to="800"
                dur="15s"
                begin="4s"
                repeatCount="indefinite"
              />
            </rect>
          </mask>

          {/* Hidden paths for rover animation */}
          <path id="path1" d="M 50 110 L 750 110" />
          <path id="path2" d="M 50 210 L 750 210" />
          <path id="path3" d="M 50 340 L 750 340" />
        </defs>
        <rect width="800" height="450" fill="url(#grid)" />

        {/* Cleared paths (green base layer) */}
        <path
          d="M 50 100 L 750 100 L 750 120 L 50 120 Z"
          fill="#22c55e"
          opacity="0.3"
          stroke="#22c55e"
          strokeWidth="2"
        />
        <path
          d="M 50 200 L 750 200 L 750 220 L 50 220 Z"
          fill="#22c55e"
          opacity="0.3"
          stroke="#22c55e"
          strokeWidth="2"
        />
        <path
          d="M 50 330 L 750 330 L 750 350 L 50 350 Z"
          fill="#22c55e"
          opacity="0.3"
          stroke="#22c55e"
          strokeWidth="2"
        />

        {/* Snow overlay (white) - disappears as rovers pass */}
        <path
          d="M 50 100 L 750 100 L 750 120 L 50 120 Z"
          fill="#ffffff"
          opacity="0.7"
          mask="url(#clearMask1)"
        />
        <path
          d="M 50 200 L 750 200 L 750 220 L 50 220 Z"
          fill="#ffffff"
          opacity="0.7"
          mask="url(#clearMask2)"
        />
        <path
          d="M 50 330 L 750 330 L 750 350 L 50 350 Z"
          fill="#ffffff"
          opacity="0.7"
          mask="url(#clearMask3)"
        />

        {/* Snow texture (small dots for visual interest) */}
        <g opacity="0.4" mask="url(#clearMask1)">
          {[...Array(30)].map((_, i) => (
            <circle
              key={`snow1-${i}`}
              cx={50 + (i * 700) / 30 + Math.random() * 10}
              cy={105 + Math.random() * 10}
              r="2"
              fill="#ffffff"
            />
          ))}
        </g>
        <g opacity="0.4" mask="url(#clearMask2)">
          {[...Array(30)].map((_, i) => (
            <circle
              key={`snow2-${i}`}
              cx={50 + (i * 700) / 30 + Math.random() * 10}
              cy={205 + Math.random() * 10}
              r="2"
              fill="#ffffff"
            />
          ))}
        </g>
        <g opacity="0.4" mask="url(#clearMask3)">
          {[...Array(30)].map((_, i) => (
            <circle
              key={`snow3-${i}`}
              cx={50 + (i * 700) / 30 + Math.random() * 10}
              cy={335 + Math.random() * 10}
              r="2"
              fill="#ffffff"
            />
          ))}
        </g>

        {/* Cross streets (cleared) */}
        <path
          d="M 250 100 L 250 350"
          stroke="#22c55e"
          strokeWidth="20"
          opacity="0.3"
        />
        <path
          d="M 550 100 L 550 350"
          stroke="#22c55e"
          strokeWidth="20"
          opacity="0.3"
        />

        {/* Rover positions (animated along paths) */}
        <RoverIcon x={200} y={110} color="#ff6600" pathId="path1" />
        <RoverIcon x={500} y={210} color="#ff6600" pathId="path2" />
        <RoverIcon x={650} y={340} color="#ff6600" pathId="path3" />

        {/* Labels */}
        <text
          x="400"
          y="430"
          textAnchor="middle"
          fontSize="14"
          fill="currentColor"
          opacity="0.6"
          fontFamily="Berkeley Mono, monospace"
        >
          50 miles cleared autonomously
        </text>
      </svg>
    </div>
  );
}

function RoverIcon({
  x,
  y,
  color,
  pathId,
}: {
  x: number;
  y: number;
  color: string;
  pathId?: string;
}) {
  // Calculate delay based on initial x position for staggered start
  const delay = `${(x / 200) * 2}s`;

  return (
    <g className="rover-icon" opacity="0">
      {/* Rover body */}
      <rect
        x={-10}
        y={-5}
        width="20"
        height="10"
        fill={color}
        rx="2"
      />
      {/* Direction indicator */}
      <circle cx={12} cy={0} r="3" fill={color} opacity="0.7" />

      {/* Pulsing safety bubble */}
      <circle
        cx={0}
        cy={0}
        r="25"
        fill="none"
        stroke={color}
        strokeWidth="1"
        opacity="0.2"
      >
        <animate
          attributeName="r"
          values="25;30;25"
          dur="2s"
          repeatCount="indefinite"
        />
        <animate
          attributeName="opacity"
          values="0.2;0.1;0.2"
          dur="2s"
          repeatCount="indefinite"
        />
      </circle>

      {/* Fade in when animation starts */}
      <animate
        attributeName="opacity"
        from="0"
        to="1"
        dur="0.5s"
        begin={delay}
        fill="freeze"
      />

      {/* Animate along path */}
      {pathId && (
        <animateMotion
          dur="15s"
          repeatCount="indefinite"
          begin={delay}
        >
          <mpath href={`#${pathId}`} />
        </animateMotion>
      )}
    </g>
  );
}
