import { ReactNode } from "react";

interface CardProps {
  title?: ReactNode;
  highlight?: boolean;
  id?: string;
  className?: string;
  children: ReactNode;
}

export function Card({ title, highlight, id, className, children }: CardProps) {
  return (
    <section
      id={id}
      className={`card ${highlight ? "highlight" : ""} ${className || ""}`}
    >
      {title && <div className="card-title">{title}</div>}
      <div className="card-body">{children}</div>
    </section>
  );
}
