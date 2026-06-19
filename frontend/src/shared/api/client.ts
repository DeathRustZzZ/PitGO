/**
 * Тонкий типизированный HTTP-клиент.
 *
 * Сейчас фронт работает на моках (см. api-модули внутри features), поэтому
 * клиент почти не используется. Он заложен заранее, чтобы при готовности
 * бэкенда переключение с мока на реальный API было локальным: меняется
 * реализация в feature-api, а не вызывающий код.
 */

const BASE_URL = import.meta.env.VITE_API_BASE_URL ?? "/api";

/** Ошибка уровня приложения с человекочитаемым сообщением. */
export class ApiError extends Error {
  readonly status: number;

  constructor(message: string, status: number) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export async function apiFetch<T>(
  path: string,
  init?: RequestInit,
): Promise<T> {
  const response = await fetch(`${BASE_URL}${path}`, {
    headers: { "Content-Type": "application/json", ...init?.headers },
    ...init,
  });

  if (!response.ok) {
    throw new ApiError(
      `Запрос завершился с ошибкой ${response.status}`,
      response.status,
    );
  }

  return (await response.json()) as T;
}
