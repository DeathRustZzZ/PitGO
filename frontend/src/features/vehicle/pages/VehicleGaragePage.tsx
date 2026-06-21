import { useState, useEffect } from "react";
import { Link, useNavigate } from "react-router-dom";
import { Button, Logo, Badge } from "../../../shared/ui";
import { getVehicles } from "../api/vehicleApi";
import type { Vehicle } from "../api/types";
import styles from "./VehicleGaragePage.module.css";

export function VehicleGaragePage() {
  const navigate = useNavigate();
  const [vehicles, setVehicles] = useState<Vehicle[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    getVehicles("demo")
      .then(setVehicles)
      .finally(() => setLoading(false));
  }, []);

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerInner}>
          <Link to="/" aria-label="На главную">
            <Logo />
          </Link>
          <Button
            variant="secondary"
            size="sm"
            onClick={() => navigate("/vehicles/new")}
          >
            + Добавить автомобиль
          </Button>
        </div>
      </header>

      <main className={styles.main}>
        <div className={styles.titleRow}>
          <h1 className={styles.title}>Мои автомобили</h1>
        </div>

        {loading && <p className={styles.empty}>Загрузка...</p>}

        {!loading && vehicles.length === 0 && (
          <div className={styles.empty}>
            <p className={styles.emptyTitle}>У вас пока нет автомобилей</p>
            <p className={styles.emptyText}>
              Добавьте первый автомобиль, чтобы записываться на обслуживание
            </p>
            <Button onClick={() => navigate("/vehicles/new")}>
              Добавить автомобиль
            </Button>
          </div>
        )}

        {!loading && vehicles.length > 0 && (
          <div className={styles.grid}>
            {vehicles.map((v) => (
              <div key={v.id} className={styles.vehicleCard}>
                <div className={styles.cardHeader}>
                  <div>
                    <p className={styles.carName}>
                      {v.make} {v.model} {v.year}
                    </p>
                    <p className={styles.plateNumber}>{v.plateNumber}</p>
                  </div>
                  <Badge
                    tone={v.status === "active" ? "success" : "neutral"}
                    dot
                  >
                    {v.status === "active" ? "Активен" : "В архиве"}
                  </Badge>
                </div>
                <div className={styles.meta}>
                  {v.mileage && (
                    <p className={styles.metaRow}>
                      Пробег: {v.mileage.toLocaleString("ru-RU")} км
                    </p>
                  )}
                  {v.vin && <p className={styles.metaRow}>VIN: {v.vin}</p>}
                </div>
              </div>
            ))}
          </div>
        )}
      </main>
    </div>
  );
}
