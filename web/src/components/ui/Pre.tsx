import { ReactNode } from "react";

interface PreProps {
  children: ReactNode;
  className?: string;
}

/**
 * Pre-formatted text component that suppresses hydration warnings.
 * Used for content mixing template literals with JSX elements,
 * where whitespace differences between SSR and client are expected.
 */
export function Pre({ children, className }: PreProps) {
  return (
    <pre className={className} suppressHydrationWarning>
      {children}
    </pre>
  );
}
