import type { ReactNode } from "react";

interface WhyNowCardProps {
  icon: ReactNode;
  title: string;
  points: string[];
}

export function WhyNowCard({ icon, title, points }: WhyNowCardProps) {
  return (
    <div className="why-now-card">
      <div className="why-now-card-header">
        <div className="why-now-card-icon">{icon}</div>
        <h3 className="why-now-card-title">{title}</h3>
      </div>
      <ul className="why-now-card-points">
        {points.map((point, i) => (
          <li key={i}>{point}</li>
        ))}
      </ul>
    </div>
  );
}
