import { useParams, Link, useNavigate } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { ordersApi, clientsApi, carsApi } from "@/shared/api/mock/store";
import type { OrderStatus, OrderWork, OrderPart } from "@/shared/types";
import { OrderStatusBadge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { Dialog, DialogContent, DialogClose } from "@/shared/ui/dialog";
import { Input } from "@/shared/ui/input";
import { formatDate, formatPrice } from "@/shared/lib/utils";
import { ArrowLeft, Edit, Plus, Trash2, CheckCircle } from "lucide-react";

const statusFlow: Record<OrderStatus, OrderStatus | null> = {
  new: "diagnostics",
  diagnostics: "in_progress",
  waiting_parts: "in_progress",
  in_progress: "ready",
  ready: "completed",
  completed: null,
  cancelled: null,
};

const nextStatusLabel: Partial<Record<OrderStatus, string>> = {
  new: "Начать диагностику",
  diagnostics: "Взять в работу",
  waiting_parts: "Взять в работу",
  in_progress: "Готово к выдаче",
  ready: "Выдать клиенту",
};

type WorkForm = { name: string; price: string; quantity: string };
type PartForm = { name: string; sku: string; price: string; quantity: string };

export function OrderDetailPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();

  const [addWorkOpen, setAddWorkOpen] = useState(false);
  const [addPartOpen, setAddPartOpen] = useState(false);
  const [workForm, setWorkForm] = useState<WorkForm>({
    name: "",
    price: "",
    quantity: "1",
  });
  const [partForm, setPartForm] = useState<PartForm>({
    name: "",
    sku: "",
    price: "",
    quantity: "1",
  });

  const { data: order, isLoading } = useQuery({
    queryKey: ["orders", id],
    queryFn: () => ordersApi.get(id!),
  });
  const { data: clients = [] } = useQuery({
    queryKey: ["clients"],
    queryFn: clientsApi.list,
  });
  const { data: cars = [] } = useQuery({
    queryKey: ["cars"],
    queryFn: carsApi.list,
  });

  const invalidate = () => {
    void qc.invalidateQueries({ queryKey: ["orders"] });
  };

  const statusMutation = useMutation({
    mutationFn: (status: OrderStatus) => ordersApi.updateStatus(id!, status),
    onSuccess: invalidate,
  });

  const updateMutation = useMutation({
    mutationFn: (data: Parameters<typeof ordersApi.update>[1]) =>
      ordersApi.update(id!, data),
    onSuccess: invalidate,
  });

  if (isLoading) return <p className="text-[var(--color-hint)]">Загрузка...</p>;
  if (!order) return <p className="text-red-500">Заказ не найден</p>;

  const client = clients.find((c) => c.id === order.clientId);
  const car = cars.find((c) => c.id === order.carId);

  const nextStatus = statusFlow[order.status];

  const handleAddWork = () => {
    if (!workForm.name || !workForm.price) return;
    const newWork: OrderWork = {
      id: String(Date.now()),
      name: workForm.name,
      price: Number(workForm.price),
      quantity: Number(workForm.quantity) || 1,
    };
    void updateMutation.mutateAsync({ works: [...order.works, newWork] });
    setWorkForm({ name: "", price: "", quantity: "1" });
    setAddWorkOpen(false);
  };

  const handleRemoveWork = (workId: string) => {
    void updateMutation.mutateAsync({
      works: order.works.filter((w) => w.id !== workId),
    });
  };

  const handleAddPart = () => {
    if (!partForm.name || !partForm.price) return;
    const newPart: OrderPart = {
      id: String(Date.now()),
      name: partForm.name,
      sku: partForm.sku || undefined,
      price: Number(partForm.price),
      quantity: Number(partForm.quantity) || 1,
    };
    void updateMutation.mutateAsync({ parts: [...order.parts, newPart] });
    setPartForm({ name: "", sku: "", price: "", quantity: "1" });
    setAddPartOpen(false);
  };

  const handleRemovePart = (partId: string) => {
    void updateMutation.mutateAsync({
      parts: order.parts.filter((p) => p.id !== partId),
    });
  };

  const worksTotal = order.works.reduce((s, w) => s + w.price * w.quantity, 0);
  const partsTotal = order.parts.reduce((s, p) => s + p.price * p.quantity, 0);

  return (
    <div className="max-w-5xl flex flex-col gap-4">
      {/* Шапка */}
      <div className="flex items-center gap-3">
        <button
          onClick={() => navigate(-1)}
          className="p-2 rounded-[var(--radius-md)] hover:bg-[var(--color-surface-muted)] transition-colors"
        >
          <ArrowLeft size={18} className="text-[var(--color-hint)]" />
        </button>
        <div className="flex-1">
          <div className="flex items-center gap-3">
            <h2 className="text-xl font-bold text-[var(--color-text)]">
              {order.orderNumber}
            </h2>
            <OrderStatusBadge status={order.status} />
          </div>
          <p className="text-sm text-[var(--color-hint)] mt-0.5">
            Создан {formatDate(order.createdAt)}
          </p>
        </div>
        <div className="flex items-center gap-2">
          {order.status === "in_progress" && (
            <Button
              variant="outline"
              onClick={() => statusMutation.mutate("waiting_parts")}
            >
              Ожидание запчастей
            </Button>
          )}
          {nextStatus && nextStatusLabel[order.status] && (
            <Button
              onClick={() => statusMutation.mutate(nextStatus)}
              disabled={statusMutation.isPending}
              className={
                order.status === "ready"
                  ? "bg-green-600 hover:bg-green-700"
                  : ""
              }
            >
              {order.status === "ready" && <CheckCircle size={16} />}
              {nextStatusLabel[order.status]}
            </Button>
          )}
          <Link to={`/app/orders/${order.id}/edit`}>
            <Button variant="secondary">
              <Edit size={16} />
              Редактировать
            </Button>
          </Link>
        </div>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* Основная информация */}
        <div className="lg:col-span-2 flex flex-col gap-4">
          {/* Описание */}
          <Card>
            <CardHeader>
              <CardTitle>Описание проблемы</CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm text-[var(--color-text)] whitespace-pre-wrap">
                {order.description || (
                  <span className="text-[var(--color-hint)]">Не указано</span>
                )}
              </p>
            </CardContent>
          </Card>

          {/* Работы */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Работы</CardTitle>
              {order.status !== "completed" && order.status !== "cancelled" && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setAddWorkOpen(true)}
                >
                  <Plus size={14} />
                  Добавить
                </Button>
              )}
            </CardHeader>
            <CardContent className="p-0">
              {order.works.length === 0 ? (
                <p className="px-5 py-6 text-sm text-[var(--color-hint)] text-center">
                  Работы не добавлены
                </p>
              ) : (
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-[var(--color-border)] bg-[var(--color-bg)]">
                      <th className="text-left text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Наименование
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Кол-во
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Цена
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Сумма
                      </th>
                      <th className="px-2 py-2" />
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--color-border)]">
                    {order.works.map((w) => (
                      <tr key={w.id}>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-text)]">
                          {w.name}
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-hint)] text-right">
                          {w.quantity}
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-hint)] text-right">
                          {formatPrice(w.price)}
                        </td>
                        <td className="px-5 py-2.5 text-sm font-medium text-[var(--color-text)] text-right">
                          {formatPrice(w.price * w.quantity)}
                        </td>
                        <td className="px-2 py-2.5">
                          {order.status !== "completed" &&
                            order.status !== "cancelled" && (
                              <button
                                onClick={() => handleRemoveWork(w.id)}
                                className="p-1 text-[var(--color-hint)] hover:text-red-500 transition-colors"
                              >
                                <Trash2 size={14} />
                              </button>
                            )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                  <tfoot>
                    <tr className="border-t border-[var(--color-border)] bg-[var(--color-bg)]">
                      <td
                        colSpan={3}
                        className="px-5 py-2 text-sm font-medium text-[var(--color-text)] text-right"
                      >
                        Итого работы:
                      </td>
                      <td className="px-5 py-2 text-sm font-bold text-[var(--color-text)] text-right">
                        {formatPrice(worksTotal)}
                      </td>
                      <td />
                    </tr>
                  </tfoot>
                </table>
              )}
            </CardContent>
          </Card>

          {/* Запчасти */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between">
              <CardTitle>Запчасти и материалы</CardTitle>
              {order.status !== "completed" && order.status !== "cancelled" && (
                <Button
                  variant="ghost"
                  size="sm"
                  onClick={() => setAddPartOpen(true)}
                >
                  <Plus size={14} />
                  Добавить
                </Button>
              )}
            </CardHeader>
            <CardContent className="p-0">
              {order.parts.length === 0 ? (
                <p className="px-5 py-6 text-sm text-[var(--color-hint)] text-center">
                  Запчасти не добавлены
                </p>
              ) : (
                <table className="w-full">
                  <thead>
                    <tr className="border-b border-[var(--color-border)] bg-[var(--color-bg)]">
                      <th className="text-left text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Наименование
                      </th>
                      <th className="text-left text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Артикул
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Кол-во
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Цена
                      </th>
                      <th className="text-right text-xs font-medium px-5 py-2 text-[var(--color-hint)]">
                        Сумма
                      </th>
                      <th className="px-2 py-2" />
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-[var(--color-border)]">
                    {order.parts.map((p) => (
                      <tr key={p.id}>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-text)]">
                          {p.name}
                        </td>
                        <td className="px-5 py-2.5 text-xs text-[var(--color-hint)]">
                          {p.sku ?? "—"}
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-hint)] text-right">
                          {p.quantity}
                        </td>
                        <td className="px-5 py-2.5 text-sm text-[var(--color-hint)] text-right">
                          {formatPrice(p.price)}
                        </td>
                        <td className="px-5 py-2.5 text-sm font-medium text-[var(--color-text)] text-right">
                          {formatPrice(p.price * p.quantity)}
                        </td>
                        <td className="px-2 py-2.5">
                          {order.status !== "completed" &&
                            order.status !== "cancelled" && (
                              <button
                                onClick={() => handleRemovePart(p.id)}
                                className="p-1 text-[var(--color-hint)] hover:text-red-500 transition-colors"
                              >
                                <Trash2 size={14} />
                              </button>
                            )}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                  <tfoot>
                    <tr className="border-t border-[var(--color-border)] bg-[var(--color-bg)]">
                      <td
                        colSpan={4}
                        className="px-5 py-2 text-sm font-medium text-[var(--color-text)] text-right"
                      >
                        Итого запчасти:
                      </td>
                      <td className="px-5 py-2 text-sm font-bold text-[var(--color-text)] text-right">
                        {formatPrice(partsTotal)}
                      </td>
                      <td />
                    </tr>
                  </tfoot>
                </table>
              )}
            </CardContent>
          </Card>
        </div>

        {/* Сайдбар */}
        <div className="flex flex-col gap-4">
          {/* Итог */}
          <Card className="border-[var(--color-border)]">
            <CardContent className="py-5">
              <p className="text-sm text-[var(--color-hint)] mb-1">
                Итоговая сумма
              </p>
              <p className="text-3xl font-bold text-[var(--color-text)]">
                {formatPrice(order.totalPrice)}
              </p>
              <div className="mt-3 flex flex-col gap-1 text-xs text-[var(--color-hint)] border-t border-[var(--color-border)] pt-3">
                <div className="flex justify-between">
                  <span>Работы</span>
                  <span>{formatPrice(worksTotal)}</span>
                </div>
                <div className="flex justify-between">
                  <span>Запчасти</span>
                  <span>{formatPrice(partsTotal)}</span>
                </div>
              </div>
            </CardContent>
          </Card>

          {/* Клиент */}
          <Card>
            <CardHeader>
              <CardTitle>Клиент</CardTitle>
            </CardHeader>
            <CardContent>
              {client ? (
                <div>
                  <Link
                    to={`/app/clients/${client.id}`}
                    className="text-sm font-medium text-[var(--color-link)] hover:underline"
                  >
                    {client.fullName}
                  </Link>
                  <p className="text-sm text-[var(--color-hint)] mt-1">
                    {client.phone}
                  </p>
                  {client.email && (
                    <p className="text-xs text-[var(--color-hint)] mt-0.5">
                      {client.email}
                    </p>
                  )}
                </div>
              ) : (
                <p className="text-sm text-[var(--color-hint)]">Не указан</p>
              )}
            </CardContent>
          </Card>

          {/* Автомобиль */}
          <Card>
            <CardHeader>
              <CardTitle>Автомобиль</CardTitle>
            </CardHeader>
            <CardContent>
              {car ? (
                <div>
                  <Link
                    to={`/app/cars/${car.id}`}
                    className="text-sm font-medium text-[var(--color-link)] hover:underline"
                  >
                    {car.make} {car.model} {car.year}
                  </Link>
                  <p className="text-sm font-mono text-[var(--color-text)] mt-1">
                    {car.plateNumber}
                  </p>
                  {car.vin && (
                    <p className="text-xs text-[var(--color-hint)] mt-0.5">
                      VIN: {car.vin}
                    </p>
                  )}
                  {car.mileage && (
                    <p className="text-xs text-[var(--color-hint)] mt-0.5">
                      Пробег: {car.mileage.toLocaleString("ru-RU")} км
                    </p>
                  )}
                </div>
              ) : (
                <p className="text-sm text-[var(--color-hint)]">Не указан</p>
              )}
            </CardContent>
          </Card>

          {/* Даты */}
          <Card>
            <CardContent className="py-4 flex flex-col gap-2">
              <div>
                <p className="text-xs text-[var(--color-hint)]">Создан</p>
                <p className="text-sm text-[var(--color-text)]">
                  {formatDate(order.createdAt)}
                </p>
              </div>
              {order.completedAt && (
                <div>
                  <p className="text-xs text-[var(--color-hint)]">Завершён</p>
                  <p className="text-sm text-[var(--color-text)]">
                    {formatDate(order.completedAt)}
                  </p>
                </div>
              )}
            </CardContent>
          </Card>

          {order.status !== "completed" && order.status !== "cancelled" && (
            <Button
              variant="destructive"
              className="w-full"
              onClick={() => statusMutation.mutate("cancelled")}
            >
              Отменить заказ
            </Button>
          )}
        </div>
      </div>

      {/* Диалог: добавить работу */}
      <Dialog open={addWorkOpen} onOpenChange={setAddWorkOpen}>
        <DialogContent title="Добавить работу">
          <div className="flex flex-col gap-3">
            <Input
              label="Наименование работы"
              value={workForm.name}
              onChange={(e) =>
                setWorkForm((f) => ({ ...f, name: e.target.value }))
              }
              placeholder="Замена масла двигателя"
            />
            <div className="grid grid-cols-2 gap-3">
              <Input
                label="Цена (₽)"
                type="number"
                value={workForm.price}
                onChange={(e) =>
                  setWorkForm((f) => ({ ...f, price: e.target.value }))
                }
                placeholder="800"
              />
              <Input
                label="Количество"
                type="number"
                value={workForm.quantity}
                onChange={(e) =>
                  setWorkForm((f) => ({ ...f, quantity: e.target.value }))
                }
                min="1"
              />
            </div>
            <div className="flex gap-2 justify-end pt-2">
              <DialogClose asChild>
                <Button variant="secondary">Отмена</Button>
              </DialogClose>
              <Button onClick={handleAddWork}>Добавить</Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>

      {/* Диалог: добавить запчасть */}
      <Dialog open={addPartOpen} onOpenChange={setAddPartOpen}>
        <DialogContent title="Добавить запчасть">
          <div className="flex flex-col gap-3">
            <Input
              label="Наименование"
              value={partForm.name}
              onChange={(e) =>
                setPartForm((f) => ({ ...f, name: e.target.value }))
              }
              placeholder="Масло моторное 5W-30"
            />
            <Input
              label="Артикул (SKU)"
              value={partForm.sku}
              onChange={(e) =>
                setPartForm((f) => ({ ...f, sku: e.target.value }))
              }
              placeholder="OIL-5W30-4"
            />
            <div className="grid grid-cols-2 gap-3">
              <Input
                label="Цена (₽)"
                type="number"
                value={partForm.price}
                onChange={(e) =>
                  setPartForm((f) => ({ ...f, price: e.target.value }))
                }
                placeholder="1800"
              />
              <Input
                label="Количество"
                type="number"
                value={partForm.quantity}
                onChange={(e) =>
                  setPartForm((f) => ({ ...f, quantity: e.target.value }))
                }
                min="1"
              />
            </div>
            <div className="flex gap-2 justify-end pt-2">
              <DialogClose asChild>
                <Button variant="secondary">Отмена</Button>
              </DialogClose>
              <Button onClick={handleAddPart}>Добавить</Button>
            </div>
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
}
