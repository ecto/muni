import { type ButtonHTMLAttributes, forwardRef } from "react";

type Variant = "primary" | "secondary" | "ghost" | "danger";
type Size = "sm" | "md" | "lg";

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: Variant;
  size?: Size;
}

const variantStyles: Record<Variant, string> = {
  primary:
    "bg-orange-500 text-black hover:bg-orange-400 border-transparent",
  secondary:
    "bg-transparent text-fg border-border hover:border-fg",
  ghost:
    "bg-transparent text-fg-muted border-transparent hover:text-fg hover:bg-surface",
  danger:
    "bg-red-600 text-white border-transparent hover:bg-red-500",
};

const sizeStyles: Record<Size, string> = {
  sm: "px-3 py-1 text-xs",
  md: "px-4 py-2 text-sm",
  lg: "px-6 py-3 text-base",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ variant = "primary", size = "md", className = "", ...props }, ref) => {
    return (
      <button
        ref={ref}
        className={`
          inline-flex items-center justify-center
          font-mono font-bold
          border
          transition-[color,background-color,border-color] duration-100 ease-default
          disabled:opacity-40 disabled:pointer-events-none
          ${variantStyles[variant]}
          ${sizeStyles[size]}
          ${className}
        `}
        {...props}
      />
    );
  },
);

Button.displayName = "Button";
