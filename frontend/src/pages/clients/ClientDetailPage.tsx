import { useParams, useNavigate, Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { useState } from "react";
import { clientsApi, carsApi, ordersApi } from "@/shared/api/mock/store";
import { Button } from "@/shared/ui/button";
import { Input, Textarea } from "@/shared/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { OrderStatusBadge } from "@/shared/ui/badge";
import { Dialog, DialogContent, DialogClose } from "@/shared/ui/dialog";
import { formatDate, formatPrice } from "@/shared/lib/utils";
import { ArrowLeft, Edit, Phone, Mail, Car, Plus } from "lucide-react";

const schema = z.object({
  fullName: z.string().min(2, "Укажите имя"),
  phone: z.string().min(10, "Укажите телефон"),
  email: z.string().email("Неверный email").or(z.literal("")).optional(),
  comment: z.string().optional(),
});
type FormValues = z.infer<typeof schema>;

export function ClientDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();
  const [editOpen, setEditOpen] = useState(false);

  const { data: client } = useQuery({
    queryKey: ["clients", id],
    queryFn: () => clientsApi.get(id!),
  });
  const { data: cars = [] } = useQuery({
    queryKey: ["cars"],
    queryFn: carsApi.list,
  });
  const { data: orders = [] } = useQuery({
    queryKey: ["orders"],
    queryFn: ordersApi.list,
  });

  const clientCars = cars.filter((c) => c.clientId === id);
  const clientOrders = orders
    .filter((o) => o.clientId === id)
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
    values: client
      ? {
          fullName: client.fullName,
          phone: client.phone,
          email: client.email ?? "",
          comment: client.comment ?? "",
        }
      : undefined,
  });

  const updateMutation = useMutation({
    mutationFn: (data: FormValues) =>
      clientsApi.update(id!, {
        fullName: data.fullName,
        phone: data.phone,
        email: data.email || undefined,
        comment: data.comment || undefined,
      }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["clients"] });
      setEditOpen(false);
    },
  });

  if (!client)
    return <p className="text-[var(--color-hint)] p-4">Загрузка...</p>;

  return (
    <div className="max-w-5xl flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <button
          onClick={() => navigate(-1)}
          className="p-2 rounded-[var(--radius-md)] hover:bg-[var(--color-surface-muted)] transition-colors"
        >
          <ArrowLeft size={18} className="text-[var(--color-hint)]" />
        </button>
        <div className="flex-1">
          <h2 className="text-xl font-bold text-[var(--color-text)]">
            {client.fullName}
          </h2>
          <p className="text-sm text-[var(--color-hint)]">
            Клиент с {formatDate(client.createdAt)}
          </p>
        </div>
        <Button variant="secondary" onClick={() => setEditOpen(true)}>
          <Edit size={16} />
          Редактировать
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <div className="lg:col-span-2 flex flex-col gap-4">
          {/* Автомобили */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Автомобили ({clientCars.length})</CardTitle>
              <Link to={`/cars/new?clientId=${id}`}>
                <Button variant="ghost" size="sm">
                  <Plus size={14} />
                  Добавить
                </Button>
              </Link>
            </CardHeader>
            <CardContent className="p-0">
              {clientCars.length === 0 ? (
                <p className="px-5 py-8 text-sm text-[var(--color-hint)] text-center">
                  Нет автомобилей
                </p>
              ) : (
                <div className="divide-y divide-[var(--color-border)]">
                  {clientCars.map((car) => (
                    <Link
                      key={car.id}
                      to={`/app/cars/${car.id}`}
                      className="flex items-center gap-3 px-5 py-3 hover:bg-[var(--color-bg)] transition-colors"
                    >
                      <Car
                        size={16}
                        className="text-[var(--color-hint)] flex-shrink-0"
                      />
                      <div className="flex-1">
                        <p className="text-sm font-medium text-[var(--color-text)]">
                          {car.make} {car.model} {car.year}
                        </p>
                        <p className="text-xs text-[var(--color-hint)] font-mono">
                          {car.plateNumber}
                        </p>
                      </div>
                      {car.mileage && (
                        <p className="text-xs text-[var(--color-hint)]">
                          {car.mileage.toLocaleString("ru-RU")} км
                        </p>
                      )}
                    </Link>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>

          {/* История заказов */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>История заказов ({clientOrders.length})</CardTitle>
              <Link to={`/orders/new`}>
                <Button variant="ghost" size="sm">
                  <Plus size={14} />
                  Новый заказ
                </Button>
              </Link>
            </CardHeader>
            <CardContent className="p-0">
              {clientOrders.length === 0 ? (
                <p className="px-5 py-8 text-sm text-[var(--color-hint)] text-center">
                  Нет заказов
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
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Сумма
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Дата
                      </th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--color-border)]">
                    {clientOrders.map((o) => (
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

        {/* Контакты */}
        <Card className="h-fit">
          <CardHeader>
            <CardTitle>Контакты</CardTitle>
          </CardHeader>
          <CardContent className="flex flex-col gap-3">
            <a
              href={`tel:${client.phone}`}
              className="flex items-center gap-2 text-sm text-[var(--color-text)] hover:text-[var(--color-link)]"
            >
              <Phone size={15} className="text-[var(--color-hint)]" />
              {client.phone}
            </a>
            {client.email && (
              <a
                href={`mailto:${client.email}`}
                className="flex items-center gap-2 text-sm text-[var(--color-text)] hover:text-[var(--color-link)]"
              >
                <Mail size={15} className="text-[var(--color-hint)]" />
                {client.email}
              </a>
            )}
            {client.comment && (
              <div className="mt-2 pt-2 border-t border-[var(--color-border)]">
                <p className="text-xs text-[var(--color-hint)] mb-1">
                  Комментарий
                </p>
                <p className="text-sm text-[var(--color-text)]">
                  {client.comment}
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      </div>

      <Dialog open={editOpen} onOpenChange={setEditOpen}>
        <DialogContent title="Редактировать клиента">
          <form
            onSubmit={handleSubmit((d) => updateMutation.mutate(d))}
            className="flex flex-col gap-3"
          >
            <Input
              id="fullName"
              label="ФИО *"
              error={errors.fullName?.message}
              {...register("fullName")}
            />
            <Input
              id="phone"
              label="Телефон *"
              error={errors.phone?.message}
              {...register("phone")}
            />
            <Input
              id="email"
              label="Email"
              error={errors.email?.message}
              {...register("email")}
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
