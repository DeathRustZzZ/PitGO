// Контракт данных Customer на стороне фронта.
// Согласуется с доменом backend (см. docs/domain/customer). Когда появится
// реальный API/OpenAPI — типы сверяются с ним.

/** Статус жизненного цикла клиента (MVP-подмножество). */
export type CustomerStatus = "onboarding" | "active";

/** Данные формы регистрации. */
export interface CustomerRegistrationInput {
  fullName: string;
  phone: string;
  email: string;
}

/** Представление клиента, возвращаемое после регистрации. */
export interface Customer {
  id: string;
  fullName: string;
  phone: string;
  email: string;
  status: CustomerStatus;
  createdAt: string;
}
