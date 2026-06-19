import { useId } from "react";
import styles from "./Logo.module.css";

interface LogoProps {
  /** "full" — знак + начертание; "mark" — только знак. */
  variant?: "full" | "mark";
  /** Размер знака в пикселях (квадрат). */
  size?: number;
  className?: string;
}

/**
 * Логотип PitGO.
 *
 * Знак — геометрическая монограмма «P» в плитке с фирменным градиентом
 * (синий → фиолетовый, как у CTA-блока) и штрихами движения, отсылающими к
 * pit-stop / скорости. Начертание: «Pit» цветом текста + «GO» акцентом,
 * поэтому логотип остаётся читаемым и в светлой, и в тёмной теме.
 */
export function Logo({ variant = "full", size = 34, className }: LogoProps) {
  // Уникальный id градиента, чтобы несколько логотипов на странице не
  // конфликтовали по ссылке url(#...). useId даёт ":" — он невалиден в id SVG.
  const gradientId = `pitgo-logo-${useId().replace(/:/g, "")}`;
  const wordmarkSize = `${(size / 34) * 1.25}rem`;

  const mark = (
    <svg
      className={styles.mark}
      width={size}
      height={size}
      viewBox="0 0 40 40"
      fill="none"
      role="img"
      aria-label="PitGO"
    >
      <defs>
        <linearGradient id={gradientId} x1="0" y1="0" x2="40" y2="40">
          <stop offset="0" stopColor="#2563eb" />
          <stop offset="1" stopColor="#7c3aed" />
        </linearGradient>
      </defs>
      <rect width="40" height="40" rx="11" fill={`url(#${gradientId})`} />
      {/* Штрихи движения слева от монограммы */}
      <path
        d="M7 16h4M7 21h6M7 26h4"
        stroke="#ffffff"
        strokeOpacity="0.55"
        strokeWidth="2"
        strokeLinecap="round"
      />
      {/* Монограмма P */}
      <path
        d="M17 30V11h7.5a5.75 5.75 0 0 1 0 11.5H17"
        stroke="#ffffff"
        strokeWidth="3.4"
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );

  if (variant === "mark") {
    return (
      <span className={[styles.logo, className].filter(Boolean).join(" ")}>
        {mark}
      </span>
    );
  }

  return (
    <span className={[styles.logo, className].filter(Boolean).join(" ")}>
      {mark}
      <span className={styles.wordmark} style={{ fontSize: wordmarkSize }}>
        Pit<span className={styles.accent}>GO</span>
      </span>
    </span>
  );
}
