import { cn } from "@/shared/lib/utils";
import { forwardRef } from "react";

const fieldBase: React.CSSProperties = {
  width: "100%",
  background: "var(--color-surface)",
  color: "var(--color-text)",
  border: "1px solid var(--color-border)",
  borderRadius: "var(--radius-sm)",
  fontSize: "var(--font-size-sm)",
  padding: "var(--space-2) var(--space-3)",
  outline: "none",
  transition: "border-color 0.15s ease, box-shadow 0.15s ease",
};

type InputProps = React.InputHTMLAttributes<HTMLInputElement> & {
  label?: string;
  error?: string;
};

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ className, label, error, id, style, ...props }, ref) => (
    <div className="flex flex-col gap-1">
      {label && (
        <label
          htmlFor={id}
          style={{
            fontSize: "var(--font-size-sm)",
            fontWeight: "var(--weight-medium)",
            color: "var(--color-text)",
          }}
        >
          {label}
        </label>
      )}
      <input
        ref={ref}
        id={id}
        className={cn("focus:shadow-[var(--focus-ring)]", className)}
        style={{
          ...fieldBase,
          ...(error ? { borderColor: "var(--color-danger)" } : {}),
          ...style,
        }}
        {...props}
      />
      {error && (
        <p
          style={{
            fontSize: "var(--font-size-xs)",
            color: "var(--color-danger)",
          }}
        >
          {error}
        </p>
      )}
    </div>
  ),
);
Input.displayName = "Input";

type TextareaProps = React.TextareaHTMLAttributes<HTMLTextAreaElement> & {
  label?: string;
  error?: string;
};

export const Textarea = forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className, label, error, id, style, ...props }, ref) => (
    <div className="flex flex-col gap-1">
      {label && (
        <label
          htmlFor={id}
          style={{
            fontSize: "var(--font-size-sm)",
            fontWeight: "var(--weight-medium)",
            color: "var(--color-text)",
          }}
        >
          {label}
        </label>
      )}
      <textarea
        ref={ref}
        id={id}
        className={cn(
          "resize-y min-h-[80px] focus:shadow-[var(--focus-ring)]",
          className,
        )}
        style={{
          ...fieldBase,
          ...(error ? { borderColor: "var(--color-danger)" } : {}),
          ...style,
        }}
        {...props}
      />
      {error && (
        <p
          style={{
            fontSize: "var(--font-size-xs)",
            color: "var(--color-danger)",
          }}
        >
          {error}
        </p>
      )}
    </div>
  ),
);
Textarea.displayName = "Textarea";

type SelectProps = React.SelectHTMLAttributes<HTMLSelectElement> & {
  label?: string;
  error?: string;
};

export const Select = forwardRef<HTMLSelectElement, SelectProps>(
  ({ className, label, error, id, children, style, ...props }, ref) => (
    <div className="flex flex-col gap-1">
      {label && (
        <label
          htmlFor={id}
          style={{
            fontSize: "var(--font-size-sm)",
            fontWeight: "var(--weight-medium)",
            color: "var(--color-text)",
          }}
        >
          {label}
        </label>
      )}
      <select
        ref={ref}
        id={id}
        className={cn("focus:shadow-[var(--focus-ring)]", className)}
        style={{
          ...fieldBase,
          ...(error ? { borderColor: "var(--color-danger)" } : {}),
          ...style,
        }}
        {...props}
      >
        {children}
      </select>
      {error && (
        <p
          style={{
            fontSize: "var(--font-size-xs)",
            color: "var(--color-danger)",
          }}
        >
          {error}
        </p>
      )}
    </div>
  ),
);
Select.displayName = "Select";
