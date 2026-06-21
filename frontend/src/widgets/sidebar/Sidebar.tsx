import { NavLink, Link } from "react-router-dom";
import {
  LayoutDashboard,
  Users,
  Car,
  ClipboardList,
  Package,
  Bell,
} from "lucide-react";
import { Logo } from "@/shared/ui/Logo/Logo";
import styles from "./Sidebar.module.css";

const navItems = [
  { to: "/app", label: "Дашборд", icon: LayoutDashboard, end: true },
  { to: "/app/orders", label: "Заказ-наряды", icon: ClipboardList },
  { to: "/app/clients", label: "Клиенты", icon: Users },
  { to: "/app/cars", label: "Автомобили", icon: Car },
  { to: "/app/inventory", label: "Склад", icon: Package },
  { to: "/app/reminders", label: "Напоминания", icon: Bell },
];

export function Sidebar() {
  return (
    <aside className={styles.sidebar}>
      <Link to="/app" className={styles.logo}>
        <Logo variant="mark" size={30} />
        <span className={styles.logoText}>PitGO</span>
      </Link>

      <nav className={styles.nav}>
        {navItems.map(({ to, label, icon: Icon, end }) => (
          <NavLink
            key={to}
            to={to}
            end={end}
            className={({ isActive }) =>
              [styles.navLink, isActive ? styles.navLinkActive : ""].join(" ")
            }
          >
            <Icon size={15} />
            {label}
          </NavLink>
        ))}
      </nav>

      <div className={styles.footer}>
        <p className={styles.footerText}>Автосервис v1.0</p>
      </div>
    </aside>
  );
}
