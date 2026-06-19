import type { HTMLAttributes } from "react";
import styles from "./Card.module.css";

type CardProps = HTMLAttributes<HTMLDivElement>;

export function Card({ className, children, ...rest }: CardProps) {
  return (
    <div
      className={[styles.card, className ?? ""].filter(Boolean).join(" ")}
      {...rest}
    >
      {children}
    </div>
  );
}
