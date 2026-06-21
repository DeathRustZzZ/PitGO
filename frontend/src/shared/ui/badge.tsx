import type { OrderStatus } from "@/shared/types";
import { cn } from "@/shared/lib/utils";

const statusConfig: Record<
  OrderStatus,
  { label: string; color: string; bg: string }
> = {
  new: {
    label: "Новый",
    color: "var(--color-hint)",
    bg: "var(--color-bg-subtle)",
  },
  diagnostics: {
    label: "Диагностика",
    color: "var(--color-link)",
    bg: "var(--color-primary-soft)",
  },
  waiting_parts: {
    label: "Ожидание запчастей",
    color: "#c2410c",
    bg: "color-mix(in srgb, #f97316 14%, transparent)",
  },
  in_progress: {
    label: "В работе",
    color: "#6d28d9",
    bg: "color-mix(in srgb, #8b5cf6 12%, transparent)",
  },
  ready: {
    label: "Готов к выдаче",
    color: "var(--color-success)",
    bg: "color-mix(in srgb, var(--color-success) 12%, transparent)",
  },
  completed: {
    label: "Завершён",
    color: "#15803d",
    bg: "color-mix(in srgb, #15803d 12%, transparent)",
  },
  cancelled: {
    label: "Отменён",
    color: "var(--color-danger)",
    bg: "color-mix(in srgb, var(--color-danger) 10%, transparent)",
  },
};

type StatusProps = { status: OrderStatus; className?: string };

export function OrderStatusBadge({ status, className }: StatusProps) {
  const cfg = statusConfig[status];
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap",
        className,
      )}
      style={{ color: cfg.color, background: cfg.bg }}
    >
      {cfg.label}
    </span>
  );
}

type BadgeVariant = "default" | "warning" | "destructive" | "success";

const badgeVars: Record<BadgeVariant, { color: string; bg: string }> = {
  default: { color: "var(--color-hint)", bg: "var(--color-bg-subtle)" },
  warning: {
    color: "#c2410c",
    bg: "color-mix(in srgb, #f97316 14%, transparent)",
  },
  destructive: {
    color: "var(--color-danger)",
    bg: "color-mix(in srgb, var(--color-danger) 10%, transparent)",
  },
  success: {
    color: "var(--color-success)",
    bg: "color-mix(in srgb, var(--color-success) 12%, transparent)",
  },
};

export function Badge({
  children,
  variant = "default",
  className,
}: {
  children: React.ReactNode;
  variant?: BadgeVariant;
  className?: string;
}) {
  const v = badgeVars[variant];
  return (
    <span
      className={cn(
        "inline-flex items-center rounded-full px-2.5 py-0.5 text-xs font-medium whitespace-nowrap",
        className,
      )}
      style={{ color: v.color, background: v.bg }}
    >
      {children}
    </span>
  );
}
