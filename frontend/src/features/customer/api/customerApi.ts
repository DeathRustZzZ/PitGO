/**
 * МОК customer API.
 *
 * Имитирует сетевую задержку и один доменный сценарий ошибки (занятый email),
 * чтобы экран можно было довести до готовности без бэкенда. Когда появится
 * реальный эндпоинт — здесь меняется только тело функции (на apiFetch),
 * сигнатура и вызывающий код остаются прежними.
 */

import { ApiError } from "../../../shared/api/client";
import type { Customer, CustomerRegistrationInput } from "./types";

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

// Заранее «занятые» email — для демонстрации состояния ошибки.
const TAKEN_EMAILS = new Set(["taken@pitgo.app"]);

export async function registerCustomer(
  input: CustomerRegistrationInput,
): Promise<Customer> {
  await delay(700);

  if (TAKEN_EMAILS.has(input.email.toLowerCase())) {
    throw new ApiError("Этот email уже зарегистрирован", 409);
  }

  return {
    id: crypto.randomUUID(),
    fullName: input.fullName,
    phone: input.phone,
    email: input.email,
    status: "onboarding",
    createdAt: new Date().toISOString(),
  };
}
