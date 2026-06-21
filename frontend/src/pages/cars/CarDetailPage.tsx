import { useParams, useNavigate, Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { useState } from "react";
import { carsApi, clientsApi, ordersApi } from "@/shared/api/mock/store";
import { Button } from "@/shared/ui/button";
import { Input, Textarea } from "@/shared/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { OrderStatusBadge } from "@/shared/ui/badge";
import { Dialog, DialogContent, DialogClose } from "@/shared/ui/dialog";
import { formatDate, formatPrice } from "@/shared/lib/utils";
import { ArrowLeft, Edit } from "lucide-react";

const schema = z.object({
  plateNumber: z.string().min(1, "Укажите номер"),
  make: z.string().min(1, "Укажите марку"),
  model: z.string().min(1, "Укажите модель"),
  year: z.number().min(1950).max(2030),
  vin: z.string().optional(),
  mileage: z.number().optional(),
  comment: z.string().optional(),
});
type FormValues = z.infer<typeof schema>;

export function CarDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [editOpen, setEditOpen] = useState(false);

  const { data: car } = useQuery({
    queryKey: ["cars", id],
    queryFn: () => carsApi.get(id!),
  });
  const { data: clients = [] } = useQuery({
    queryKey: ["clients"],
    queryFn: clientsApi.list,
  });
  const { data: orders = [] } = useQuery({
    queryKey: ["orders"],
    queryFn: ordersApi.list,
  });

  const client = clients.find((c) => c.id === car?.clientId);
  const carOrders = orders
    .filter((o) => o.carId === id)
    .sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    );

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    values: car
      ? {
          plateNumber: car.plateNumber,
          make: car.make,
          model: car.model,
          year: car.year,
          vin: car.vin ?? "",
          mileage: car.mileage,
          comment: car.comment ?? "",
        }
      : undefined,
  });

  const updateMutation = useMutation({
    mutationFn: (data: FormValues) =>
      carsApi.update(id!, {
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
      setEditOpen(false);
    },
  });

  if (!car) return <p className="text-[var(--color-hint)]">Загрузка...</p>;

  return (
    <div className="max-w-4xl flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <button
          onClick={() => navigate(-1)}
          className="p-2 rounded-[var(--radius-md)] hover:bg-[var(--color-surface-muted)] transition-colors"
        >
          <ArrowLeft size={18} className="text-[var(--color-hint)]" />
        </button>
        <div className="flex-1">
          <h2 className="text-xl font-bold text-[var(--color-text)]">
            {car.make} {car.model} {car.year}
          </h2>
          <p className="text-sm font-mono text-[var(--color-hint)]">
            {car.plateNumber}
          </p>
        </div>
        <Button variant="secondary" onClick={() => setEditOpen(true)}>
          <Edit size={16} />
          Редактировать
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="lg:col-span-2">
          <Card>
            <CardHeader>
              <CardTitle>История ремонтов ({carOrders.length})</CardTitle>
            </CardHeader>
            <CardContent className="p-0">
              {carOrders.length === 0 ? (
                <p className="px-5 py-8 text-sm text-[var(--color-hint)] text-center">
                  Нет ремонтов
                </p>
              ) : (
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-[var(--color-border)] bg-[var(--color-bg)]">
                      <th className="text-left text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Номер
                      </th>
                      <th className="text-left text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Статус
                      </th>
                      <th className="text-left text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Описание
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Сумма
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Дата
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--color-border)]">
                    {carOrders.map((o) => (
                      <tr key={o.id} className="hover:bg-[var(--color-bg)]">
                        <td className="px-5 py-2.5">
                          <Link
                            to={`/app/orders/${o.id}`}
                            className="text-sm text-[var(--color-link)] hover:underline font-medium"
                          >
                            {o.orderNumber}
                          </Link>
                        </td>
                        <td className="px-5 py-2.5">
                          <OrderStatusBadge status={o.status} />
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-hint)] max-w-[200px] truncate">
                          {o.description}
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-text)] text-right">
                          {o.totalPrice > 0 ? formatPrice(o.totalPrice) : "—"}
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-hint)] text-right">
                          {formatDate(o.createdAt)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </CardContent>
          </Card>
        </div>

        <Card className="h-fit">
          <CardHeader>
            <CardTitle>Данные авто</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-2.5">
            <Row label="Марка" value={car.make} />
            <Row label="Модель" value={car.model} />
            <Row label="Год" value={String(car.year)} />
            <Row
              label="Гос. номер"
              value={<span className="font-mono">{car.plateNumber}</span>}
            />
            {car.vin && (
              <Row
                label="VIN"
                value={<span className="font-mono text-xs">{car.vin}</span>}
              />
            )}
            {car.mileage && (
              <Row
                label="Пробег"
                value={`${car.mileage.toLocaleString("ru-RU")} км`}
              />
            )}
            {car.comment && <Row label="Комментарий" value={car.comment} />}
            {client && (
              <div className="pt-2 border-t border-[var(--color-border)]">
                <p className="text-xs text-[var(--color-hint)] mb-1">
                  Владелец
                </p>
                <Link
                  to={`/app/clients/${client.id}`}
                  className="text-sm text-[var(--color-link)] hover:underline"
                >
                  {client.fullName}
                </Link>
                <p className="text-xs text-[var(--color-hint)] mt-0.5">
                  {client.phone}
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent title="Редактировать автомобиль">
          <form
            onSubmit={handleSubmit((d) => updateMutation.mutate(d))}
            className="flex flex-col gap-3"
          >
            <div className="grid grid-cols-2 gap-3">
              <Input
                id="make"
                label="Марка *"
                error={errors.make?.message}
                {...register("make")}
              />
              <Input
                id="model"
                label="Модель *"
                error={errors.model?.message}
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
                {...register("plateNumber")}
              />
            </div>
            <Input id="vin" label="VIN" {...register("vin")} />
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
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => reset()}
                >
                  Отмена
                </Button>
              </DialogClose>
              <Button type="submit" disabled={updateMutation.isPending}>
                {updateMutation.isPending ? "Сохраняем..." : "Сохранить"}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div>
      <p className="text-xs text-[var(--color-hint)]">{label}</p>
      <p className="text-sm text-[var(--color-text)] mt-0.5">{value}</p>
    </div>
  );
}
