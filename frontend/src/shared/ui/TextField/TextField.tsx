import type { InputHTMLAttributes } from "react";
import { useId } from "react";
import styles from "./TextField.module.css";

interface TextFieldProps extends InputHTMLAttributes<HTMLInputElement> {
  label: string;
  /** Текст ошибки. Если задан — поле подсвечивается как невалидное. */
  error?: string;
  /** Подсказка под полем (показывается, когда нет ошибки). */
  hint?: string;
}

export function TextField({
  label,
  error,
  hint,
  id,
  className,
  ...rest
}: TextFieldProps) {
  const generatedId = useId();
  const inputId = id ?? generatedId;
  const descriptionId = error
    ? `${inputId}-error`
    : hint
      ? `${inputId}-hint`
      : undefined;

  const inputClasses = [styles.input, error ? styles.invalid : ""]
    .filter(Boolean)
    .join(" ");

  return (
    <div className={[styles.field, className ?? ""].filter(Boolean).join(" ")}>
      <label className={styles.label} htmlFor={inputId}>
        {label}
      </label>
      <input
        id={inputId}
        className={inputClasses}
        aria-invalid={error ? true : undefined}
        aria-describedby={descriptionId}
        {...rest}
      />
      {error ? (
        <span id={descriptionId} className={styles.error} role="alert">
          {error}
        </span>
      ) : hint ? (
        <span id={descriptionId} className={styles.hint}>
          {hint}
        </span>
      ) : null}
    </div>
  );
}
