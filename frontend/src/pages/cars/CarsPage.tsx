import { useState } from "react";
import { Link, useSearchParams } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { carsApi, clientsApi } from "@/shared/api/mock/store";
import { Button } from "@/shared/ui/button";
import { Input, Textarea, Select } from "@/shared/ui/input";
import { EmptyState } from "@/shared/ui/empty-state";
import { Dialog, DialogContent, DialogClose } from "@/shared/ui/dialog";
import { Plus, Search } from "lucide-react";

const schema = z.object({
  clientId: z.string().min(1, "Выберите клиента"),
  plateNumber: z.string().min(1, "Укажите номер"),
  make: z.string().min(1, "Укажите марку"),
  model: z.string().min(1, "Укажите модель"),
  year: z.number({ error: "Укажите год" }).min(1950).max(2030),
  vin: z.string().optional(),
  mileage: z.number().optional(),
  comment: z.string().optional(),
});
type FormValues = z.infer<typeof schema>;

export function CarsPage() {
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [searchParams] = useSearchParams();
  const qc = useQueryClient();

  const defaultClientId = searchParams.get("clientId") ?? "";

  const { data: cars = [] } = useQuery({
    queryKey: ["cars"],
    queryFn: carsApi.list,
  });
  const { data: clients = [] } = useQuery({
    queryKey: ["clients"],
    queryFn: clientsApi.list,
  });

  const clientMap = Object.fromEntries(clients.map((c) => [c.id, c]));

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: {
      clientId: defaultClientId,
      year: new Date().getFullYear(),
    },
  });

  const createMutation = useMutation({
    mutationFn: (data: FormValues) =>
      carsApi.create({
        clientId: data.clientId,
        plateNumber: data.plateNumber,
        make: data.make,
        model: data.model,
        year: data.year,
        vin: data.vin || undefined,
        mileage: data.mileage || undefined,
        comment: data.comment || undefined,
      }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["cars"] });
      setCreateOpen(false);
      reset();
    },
  });

  const filtered = cars.filter((c) => {
    if (!search) return true;
    const q = search.toLowerCase();
    const client = clientMap[c.clientId];
    return (
      c.plateNumber.toLowerCase().includes(q) ||
      c.make.toLowerCase().includes(q) ||
      c.model.toLowerCase().includes(q) ||
      c.vin?.toLowerCase().includes(q) ||
      client?.fullName.toLowerCase().includes(q)
    );
  });

  return (
    <div className="max-w-5xl flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <div className="relative flex-1">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-hint)]"
          />
          <input
            type="text"
            placeholder="Поиск по номеру, марке, VIN, клиенту..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 text-sm rounded-[var(--radius-md)] border border-[var(--color-border)] focus:outline-none focus:ring-2 focus:shadow-[var(--focus-ring)] focus:border-[var(--color-primary)]"
          />
        </div>
        <Button
          onClick={() => {
            reset({
              clientId: defaultClientId,
              year: new Date().getFullYear(),
            });
            setCreateOpen(true);
          }}
        >
          <Plus size={16} />
          Добавить авто
        </Button>
      </div>

      <div className="bg-[var(--color-surface)] rounded-[var(--radius-lg)] border border-[var(--color-border)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <EmptyState
            title="Автомобили не найдены"
            action={
              <Button onClick={() => setCreateOpen(true)}>
                <Plus size={16} />
                Добавить авто
              </Button>
            }
          />
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b border-[var(--color-border)] bg-[var(--color-bg)]">
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Номер
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Марка / Модель
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Год
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Клиент
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Пробег
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  VIN
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)]">
              {filtered.map((car) => (
                <tr
                  key={car.id}
                  className="hover:bg-[var(--color-bg)] transition-colors"
                >
                  <td className="px-4 py-3">
                    <Link
                      to={`/cars/${car.id}`}
                      className="text-sm font-mono font-medium text-[var(--color-link)] hover:underline"
                    >
                      {car.plateNumber}
                    </Link>
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-text)]">
                    {car.make} {car.model}
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-hint)]">
                    {car.year}
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-text)]">
                    {clientMap[car.clientId] ? (
                      <Link
                        to={`/clients/${car.clientId}`}
                        className="hover:underline"
                      >
                        {clientMap[car.clientId].fullName}
                      </Link>
                    ) : (
                      "—"
                    )}
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-hint)]">
                    {car.mileage
                      ? `${car.mileage.toLocaleString("ru-RU")} км`
                      : "—"}
                  </td>
                  <td className="px-4 py-3 text-xs font-mono text-[var(--color-hint)]">
                    {car.vin ?? "—"}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent title="Добавить автомобиль">
          <form
            onSubmit={handleSubmit((d) => createMutation.mutate(d))}
            className="flex flex-col gap-3"
          >
            <Select
              id="clientId"
              label="Клиент *"
              error={errors.clientId?.message}
              {...register("clientId")}
            >
              <option value="">Выберите клиента...</option>
              {clients.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.fullName}
                </option>
              ))}
            </Select>
            <div className="grid grid-cols-2 gap-3">
              <Input
                id="make"
                label="Марка *"
                error={errors.make?.message}
                placeholder="Toyota"
                {...register("make")}
              />
              <Input
                id="model"
                label="Модель *"
                error={errors.model?.message}
                placeholder="Camry"
                {...register("model")}
              />
            </div>
            <div className="grid grid-cols-2 gap-3">
              <Input
                id="year"
                label="Год *"
                type="number"
                error={errors.year?.message}
                {...register("year", { valueAsNumber: true })}
              />
              <Input
                id="plateNumber"
                label="Гос. номер *"
                error={errors.plateNumber?.message}
                placeholder="А001АА 77"
                {...register("plateNumber")}
              />
            </div>
            <Input
              id="vin"
              label="VIN"
              placeholder="XTA210990B2345678"
              {...register("vin")}
            />
            <Input
              id="mileage"
              label="Пробег (км)"
              type="number"
              {...register("mileage", { valueAsNumber: true })}
            />
            <Textarea
              id="comment"
              label="Комментарий"
              rows={2}
              {...register("comment")}
            />
            <div className="flex gap-2 justify-end pt-2">
              <DialogClose asChild>
                <Button type="button" variant="secondary">
                  Отмена
                </Button>
              </DialogClose>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending ? "Сохраняем..." : "Создать"}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
