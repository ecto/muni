import type { ReactNode } from "react";

interface CardProps {
  children: ReactNode;
  className?: string;
}

interface CardHeaderProps {
  title: string;
  badge?: ReactNode;
}

interface CardRowProps {
  label: string;
  value: ReactNode;
}

export function Card({ children, className = "" }: CardProps) {
  return (
    <div className={`border border-border ${className}`}>
      {children}
    </div>
  );
}

export function CardHeader({ title, badge }: CardHeaderProps) {
  return (
    <div className="flex items-center justify-between px-4 py-3 border-b border-border">
      <span className="text-sm font-bold">{title}</span>
      {badge}
    </div>
  );
}

export function CardBody({ children, className = "" }: CardProps) {
  return <div className={`px-4 py-3 ${className}`}>{children}</div>;
}

export function CardRow({ label, value }: CardRowProps) {
  return (
    <div className="flex justify-between py-1 text-sm">
      <span className="text-fg-muted">{label}</span>
      <span>{value}</span>
    </div>
  );
}
