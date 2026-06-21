import { useState } from "react";
import { Link, useNavigate } from "react-router-dom";
import { Button, Logo, TextField } from "../../../shared/ui";
import { createVehicle } from "../api/vehicleApi";
import { ApiError } from "../../../shared/api/client";
import styles from "./VehicleCreatePage.module.css";

export function VehicleCreatePage() {
  const navigate = useNavigate();
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const [make, setMake] = useState("");
  const [model, setModel] = useState("");
  const [year, setYear] = useState(String(new Date().getFullYear()));
  const [plateNumber, setPlateNumber] = useState("");
  const [vin, setVin] = useState("");
  const [mileage, setMileage] = useState("");

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);
    setLoading(true);

    try {
      await createVehicle("demo", {
        make: make.trim(),
        model: model.trim(),
        year: Number(year),
        plateNumber: plateNumber.trim(),
        vin: vin.trim() || undefined,
        mileage: mileage ? Number(mileage) : undefined,
      });
      navigate("/vehicles");
    } catch (err) {
      setError(
        err instanceof ApiError
          ? err.message
          : "Не удалось добавить автомобиль",
      );
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className={styles.page}>
      <header className={styles.header}>
        <div className={styles.headerInner}>
          <Link to="/" aria-label="На главную">
            <Logo />
          </Link>
        </div>
      </header>

      <main className={styles.main}>
        <h1 className={styles.title}>Добавить автомобиль</h1>

        <form className={styles.form} onSubmit={handleSubmit}>
          {error && <p className={styles.errorBanner}>{error}</p>}

          <div className={styles.row}>
            <TextField
              label="Марка *"
              placeholder="Toyota"
              value={make}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                setMake(e.target.value)
              }
              required
            />
            <TextField
              label="Модель *"
              placeholder="Camry"
              value={model}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                setModel(e.target.value)
              }
              required
            />
          </div>

          <div className={styles.row}>
            <TextField
              label="Год выпуска *"
              type="number"
              placeholder="2020"
              value={year}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                setYear(e.target.value)
              }
              required
            />
            <TextField
              label="Гос. номер *"
              placeholder="А001АА 77"
              value={plateNumber}
              onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
                setPlateNumber(e.target.value)
              }
              required
            />
          </div>

          <TextField
            label="VIN"
            placeholder="XTA210990B2345678"
            value={vin}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
              setVin(e.target.value)
            }
          />

          <TextField
            label="Пробег (км)"
            type="number"
            placeholder="85000"
            value={mileage}
            onChange={(e: React.ChangeEvent<HTMLInputElement>) =>
              setMileage(e.target.value)
            }
          />

          <div className={styles.actions}>
            <Button type="submit" fullWidth loading={loading}>
              Добавить автомобиль
            </Button>
            <Button
              variant="ghost"
              fullWidth
              onClick={() => navigate("/vehicles")}
            >
              Отмена
            </Button>
          </div>
        </form>
      </main>
    </div>
  );
}
