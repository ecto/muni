import Link from "next/link";

export function MuniLogo({ className }: { className?: string }) {
  return (
    <svg
      className={className}
      viewBox="-60 -60 880 400"
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <g
        stroke="#ff6600"
        strokeWidth="40"
        strokeLinecap="round"
        strokeLinejoin="round"
        fill="none"
      >
        <path d="M 20 240 L 20 100 C 20 50, 80 50, 100 50 C 140 50, 160 70, 160 100" />
        <path d="M 160 240 L 160 100" />
        <path d="M 160 100 C 160 50, 220 50, 240 50 C 280 50, 300 70, 300 100 L 300 240 C 300 270, 330 280, 370 280 C 410 280, 440 270, 440 240 L 440 100" />
        <path d="M 440 100 C 440 50, 500 50, 520 50 C 560 50, 600 70, 600 100 L 600 240 C 600 270, 630 280, 670 280 C 710 280, 740 270, 740 240 L 740 100" />
      </g>
      <circle cx="740" cy="30" r="28" fill="#ff6600" />
    </svg>
  );
}

interface HeaderProps {
  showSubtitle?: boolean;
}

export function Header({ showSubtitle = true }: HeaderProps) {
  return (
    <header className="site-header">
      <Link href="/" className="brand-link">
        <MuniLogo className="brand-logo" />
        <strong className="brand-name">MUNICIPAL ROBOTICS</strong>
      </Link>
      {showSubtitle && <p className="subtitle">Autonomous sidewalk maintenance</p>}
    </header>
  );
}
