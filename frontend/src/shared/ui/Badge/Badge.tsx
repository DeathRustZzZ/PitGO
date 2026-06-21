import type { ReactNode } from "react";
import styles from "./Badge.module.css";

type BadgeTone = "neutral" | "success" | "warning" | "danger" | "info";

interface BadgeProps {
  tone?: BadgeTone;
  /** Показать ведущую точку-индикатор. */
  dot?: boolean;
  children: ReactNode;
  className?: string;
}

export function Badge({
  tone = "neutral",
  dot = false,
  children,
  className,
}: BadgeProps) {
  return (
    <span
      className={[styles.badge, styles[tone], className]
        .filter(Boolean)
        .join(" ")}
    >
      {dot && <span className={styles.dot} aria-hidden="true" />}
      {children}
    </span>
  );
}
