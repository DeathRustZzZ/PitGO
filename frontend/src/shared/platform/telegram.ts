/**
 * Тонкий слой определения платформы Telegram Mini App.
 *
 * Сейчас задача минимальна: понять, что мы внутри Telegram, и корректно
 * инициализировать WebApp (ready/expand), чтобы клиент применил тему и развернул
 * окно. Полную интеграцию (MainButton, BackButton, haptics, initData) добавим,
 * когда подключим реальный API.
 *
 * Принцип: типизированный доступ к window.Telegram.WebApp без глобальных
 * деклараций, чтобы не протекать в остальной код.
 */

/** Минимальное подмножество Telegram WebApp API, которое нам нужно сейчас. */
interface TelegramWebApp {
  readonly colorScheme: "light" | "dark";
  readonly themeParams: Record<string, string>;
  ready(): void;
  expand(): void;
}

interface TelegramNamespace {
  readonly WebApp?: TelegramWebApp;
}

function readTelegram(): TelegramWebApp | undefined {
  const tg = (window as unknown as { Telegram?: TelegramNamespace }).Telegram;
  return tg?.WebApp;
}

/** Запущены ли мы внутри Telegram Mini App. */
export function isTelegram(): boolean {
  return readTelegram() !== undefined;
}

/** Текущая цветовая схема: тему Telegram или фолбэк "light" для веба. */
export function getColorScheme(): "light" | "dark" {
  return readTelegram()?.colorScheme ?? "light";
}

/**
 * Инициализация платформы. Безопасно вызывать всегда: вне Telegram это no-op.
 * Вызывается один раз при старте приложения (main.tsx).
 */
export function initPlatform(): void {
  const webApp = readTelegram();
  if (!webApp) return;

  webApp.ready();
  webApp.expand();

  // Помечаем корень — пригодится для платформенно-специфичных стилей.
  document.documentElement.dataset.platform = "telegram";
}
