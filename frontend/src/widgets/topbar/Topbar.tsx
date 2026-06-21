import { useLocation } from "react-router-dom";
import styles from "./Topbar.module.css";

const titles: Record<string, string> = {
  "/app": "Дашборд",
  "/app/orders": "Заказ-наряды",
  "/app/orders/new": "Новый заказ-наряд",
  "/app/clients": "Клиенты",
  "/app/cars": "Автомобили",
  "/app/inventory": "Склад",
  "/app/reminders": "Напоминания",
};

function getTitle(pathname: string): string {
  if (titles[pathname]) return titles[pathname];
  if (pathname.startsWith("/app/orders/") && pathname.endsWith("/edit"))
    return "Редактирование заказа";
  if (pathname.startsWith("/app/orders/")) return "Заказ-наряд";
  if (pathname.startsWith("/app/clients/")) return "Карточка клиента";
  if (pathname.startsWith("/app/cars/")) return "Карточка автомобиля";
  return "PitGO";
}

export function Topbar() {
  const { pathname } = useLocation();
  return (
    <header className={styles.topbar}>
      <h1 className={styles.title}>{getTitle(pathname)}</h1>
    </header>
  );
}
