import { cn } from "@/shared/lib/utils";
import { forwardRef } from "react";

type ButtonProps = React.ButtonHTMLAttributes<HTMLButtonElement> & {
  variant?: "primary" | "secondary" | "ghost" | "destructive" | "outline";
  size?: "sm" | "md" | "lg";
};

const base =
  "inline-flex items-center justify-center gap-2 font-medium transition-all duration-150 disabled:opacity-50 disabled:cursor-not-allowed active:translate-y-px";

const variants: Record<NonNullable<ButtonProps["variant"]>, string> = {
  primary:
    "text-white shadow-[0_2px_8px_color-mix(in_srgb,var(--color-primary)_35%,transparent)]",
  secondary: "border",
  ghost: "",
  destructive: "text-white",
  outline: "border",
};

const sizes: Record<NonNullable<ButtonProps["size"]>, string> = {
  sm: "px-3 py-1.5 text-[var(--font-size-sm)] rounded-[var(--radius-sm)]",
  md: "px-4 py-2 text-[var(--font-size-sm)] rounded-[var(--radius-sm)]",
  lg: "px-5 py-2.5 text-[var(--font-size-md)] rounded-[var(--radius-md)]",
};

export const Button = forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant = "primary", size = "md", style, ...props }, ref) => {
    const variantStyles: Record<
      NonNullable<ButtonProps["variant"]>,
      React.CSSProperties
    > = {
      primary: {
        background: "var(--color-primary)",
        color: "var(--color-on-primary)",
      },
      secondary: {
        background: "var(--color-surface)",
        color: "var(--color-text)",
        borderColor: "var(--color-border)",
      },
      ghost: {
        background: "transparent",
        color: "var(--color-link)",
      },
      destructive: {
        background: "var(--color-danger)",
        color: "#ffffff",
      },
      outline: {
        background: "transparent",
        color: "var(--color-text)",
        borderColor: "var(--color-border)",
      },
    };

    return (
      <button
        ref={ref}
        className={cn(base, variants[variant], sizes[size], className)}
        style={{ ...variantStyles[variant], ...style }}
        {...props}
      />
    );
  },
);
Button.displayName = "Button";
