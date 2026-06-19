import type { FormEvent } from "react";
import { useState } from "react";
import { Link } from "react-router-dom";
import { Button, Card, Logo, TextField } from "../../../shared/ui";
import { ApiError } from "../../../shared/api/client";
import { registerCustomer } from "../api/customerApi";
import type { Customer } from "../api/types";
import styles from "./CustomerRegistrationPage.module.css";

interface FormState {
  fullName: string;
  phone: string;
  email: string;
}

type FieldErrors = Partial<Record<keyof FormState, string>>;

const EMPTY_FORM: FormState = { fullName: "", phone: "", email: "" };

// Простая клиентская валидация. Бэкенд остаётся источником истины —
// здесь только быстрый UX-фидбек, не замена серверных инвариантов.
function validate(form: FormState): FieldErrors {
  const errors: FieldErrors = {};

  if (form.fullName.trim().length < 2) {
    errors.fullName = "Укажите имя";
  }
  if (!/^\+?\d[\d\s()-]{7,}$/.test(form.phone.trim())) {
    errors.phone = "Введите корректный номер телефона";
  }
  if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(form.email.trim())) {
    errors.email = "Введите корректный email";
  }

  return errors;
}

export function CustomerRegistrationPage() {
  const [form, setForm] = useState<FormState>(EMPTY_FORM);
  const [errors, setErrors] = useState<FieldErrors>({});
  const [formError, setFormError] = useState<string | null>(null);
  const [submitting, setSubmitting] = useState(false);
  const [created, setCreated] = useState<Customer | null>(null);

  function updateField(field: keyof FormState, value: string) {
    setForm((prev) => ({ ...prev, [field]: value }));
    // Сбрасываем ошибку поля по мере правки — меньше визуального шума.
    if (errors[field]) {
      setErrors((prev) => ({ ...prev, [field]: undefined }));
    }
  }

  async function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setFormError(null);

    const validationErrors = validate(form);
    if (Object.keys(validationErrors).length > 0) {
      setErrors(validationErrors);
      return;
    }

    setSubmitting(true);
    try {
      const customer = await registerCustomer({
        fullName: form.fullName.trim(),
        phone: form.phone.trim(),
        email: form.email.trim(),
      });
      setCreated(customer);
    } catch (error) {
      setFormError(
        error instanceof ApiError
          ? error.message
          : "Не удалось завершить регистрацию. Попробуйте ещё раз.",
      );
    } finally {
      setSubmitting(false);
    }
  }

  const brand = (
    <Link to="/" className={styles.brand} aria-label="PitGO — на главную">
      <Logo size={30} />
    </Link>
  );

  if (created) {
    return (
      <main className={styles.page}>
        {brand}
        <Card className={styles.card}>
          <div className={styles.success}>
            <div className={styles.successIcon} aria-hidden="true">
              ✓
            </div>
            <h1 className={styles.title}>Готово, {created.fullName}!</h1>
            <p className={styles.subtitle}>
              Аккаунт создан. Статус: онбординг — подтвердите контакты, чтобы
              активировать профиль.
            </p>
            <p className={styles.successMeta}>ID клиента: {created.id}</p>
            <Button
              variant="secondary"
              fullWidth
              onClick={() => {
                setCreated(null);
                setForm(EMPTY_FORM);
              }}
            >
              Зарегистрировать ещё одного
            </Button>
          </div>
        </Card>
        <Link to="/" className={styles.backLink}>
          ← На главную
        </Link>
      </main>
    );
  }

  return (
    <main className={styles.page}>
      {brand}
      <Card className={styles.card}>
        <header className={styles.header}>
          <h1 className={styles.title}>Регистрация в PitGO</h1>
          <p className={styles.subtitle}>
            Создайте аккаунт клиента, чтобы записываться в автосервисы и хранить
            историю обслуживания.
          </p>
        </header>

        <form className={styles.form} onSubmit={handleSubmit} noValidate>
          <TextField
            label="Имя"
            name="fullName"
            autoComplete="name"
            placeholder="Иван Петров"
            value={form.fullName}
            error={errors.fullName}
            onChange={(e) => updateField("fullName", e.target.value)}
          />
          <TextField
            label="Телефон"
            name="phone"
            type="tel"
            autoComplete="tel"
            placeholder="+7 999 123-45-67"
            value={form.phone}
            error={errors.phone}
            onChange={(e) => updateField("phone", e.target.value)}
          />
          <TextField
            label="Email"
            name="email"
            type="email"
            autoComplete="email"
            placeholder="ivan@example.com"
            value={form.email}
            error={errors.email}
            onChange={(e) => updateField("email", e.target.value)}
          />

          {formError && (
            <p className={styles.formError} role="alert">
              {formError}
            </p>
          )}

          <Button type="submit" fullWidth loading={submitting}>
            {submitting ? "Создаём аккаунт…" : "Зарегистрироваться"}
          </Button>

          <p className={styles.terms}>
            Регистрируясь, вы соглашаетесь с условиями использования.
          </p>
        </form>
      </Card>
      <Link to="/" className={styles.backLink}>
        ← На главную
      </Link>
    </main>
  );
}
