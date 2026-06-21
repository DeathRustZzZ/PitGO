import { Outlet } from "react-router-dom";
import { Sidebar } from "@/widgets/sidebar/Sidebar";
import { Topbar } from "@/widgets/topbar/Topbar";
import styles from "./AppLayout.module.css";

export function AppLayout() {
  return (
    <div className={styles.shell}>
      <Sidebar />
      <div className={styles.body}>
        <Topbar />
        <main className={styles.main}>
          <Outlet />
        </main>
      </div>
    </div>
  );
}
