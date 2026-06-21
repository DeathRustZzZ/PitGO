import { useState } from "react";
import { Link } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { ordersApi } from "@/shared/api/mock/store";
import { clientsApi } from "@/shared/api/mock/store";
import { carsApi } from "@/shared/api/mock/store";
import type { OrderStatus } from "@/shared/types";
import { OrderStatusBadge } from "@/shared/ui/badge";
import { Button } from "@/shared/ui/button";
import { EmptyState } from "@/shared/ui/empty-state";
import { formatDate, formatPrice } from "@/shared/lib/utils";
import { Plus, Search } from "lucide-react";

const statusOptions: { value: OrderStatus | "all"; label: string }[] = [
  { value: "all", label: "Все" },
  { value: "new", label: "Новые" },
  { value: "diagnostics", label: "Диагностика" },
  { value: "waiting_parts", label: "Ожидание запчастей" },
  { value: "in_progress", label: "В работе" },
  { value: "ready", label: "Готовы к выдаче" },
  { value: "completed", label: "Завершённые" },
  { value: "cancelled", label: "Отменённые" },
];

export function OrdersPage() {
  const [search, setSearch] = useState("");
  const [statusFilter, setStatusFilter] = useState<OrderStatus | "all">("all");

  const { data: orders = [] } = useQuery({
    queryKey: ["orders"],
    queryFn: ordersApi.list,
  });
  const { data: clients = [] } = useQuery({
    queryKey: ["clients"],
    queryFn: clientsApi.list,
  });
  const { data: cars = [] } = useQuery({
    queryKey: ["cars"],
    queryFn: carsApi.list,
  });

  const clientMap = Object.fromEntries(clients.map((c) => [c.id, c]));
  const carMap = Object.fromEntries(cars.map((c) => [c.id, c]));

  const filtered = orders
    .filter((o) => statusFilter === "all" || o.status === statusFilter)
    .filter((o) => {
      if (!search) return true;
      const q = search.toLowerCase();
      const client = clientMap[o.clientId];
      const car = carMap[o.carId];
      return (
        o.orderNumber.toLowerCase().includes(q) ||
        client?.fullName.toLowerCase().includes(q) ||
        car?.plateNumber.toLowerCase().includes(q) ||
        car?.make.toLowerCase().includes(q)
      );
    })
    .sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    );

  return (
    <div className="max-w-6xl flex flex-col gap-4">
      {/* Toolbar */}
      <div className="flex items-center gap-3 flex-wrap">
        <div className="relative flex-1 min-w-[220px]">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-hint)]"
          />
          <input
            type="text"
            placeholder="Поиск по номеру, клиенту, авто..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 text-sm rounded-[var(--radius-md)] border border-[var(--color-border)] focus:outline-none focus:ring-2 focus:shadow-[var(--focus-ring)] focus:border-[var(--color-primary)]"
          />
        </div>
        <select
          value={statusFilter}
          onChange={(e) =>
            setStatusFilter(e.target.value as OrderStatus | "all")
          }
          className="text-sm rounded-[var(--radius-md)] border border-[var(--color-border)] px-3 py-2 bg-[var(--color-surface)] focus:outline-none focus:ring-2 focus:shadow-[var(--focus-ring)] focus:border-[var(--color-primary)]"
        >
          {statusOptions.map((o) => (
            <option key={o.value} value={o.value}>
              {o.label}
            </option>
          ))}
        </select>
        <Link to="/orders/new">
          <Button>
            <Plus size={16} />
            Новый заказ
          </Button>
        </Link>
      </div>

      {/* Таблица */}
      <div className="bg-[var(--color-surface)] rounded-[var(--radius-lg)] border border-[var(--color-border)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <EmptyState
            title="Заказы не найдены"
            description="Попробуйте изменить фильтры или создайте новый заказ"
            action={
              <Link to="/orders/new">
                <Button>
                  <Plus size={16} />
                  Новый заказ
                </Button>
              </Link>
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
                  Клиент
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Автомобиль
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Статус
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Сумма
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Дата
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)]">
              {filtered.map((order) => {
                const client = clientMap[order.clientId];
                const car = carMap[order.carId];
                return (
                  <tr
                    key={order.id}
                    className="hover:bg-[var(--color-bg)] transition-colors"
                  >
                    <td className="px-4 py-3">
                      <Link
                        to={`/app/orders/${order.id}`}
                        className="text-sm font-medium text-[var(--color-link)] hover:underline"
                      >
                        {order.orderNumber}
                      </Link>
                    </td>
                    <td className="px-4 py-3 text-sm text-[var(--color-text)]">
                      {client ? (
                        <Link
                          to={`/app/clients/${client.id}`}
                          className="hover:underline"
                        >
                          {client.fullName}
                        </Link>
                      ) : (
                        "—"
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-[var(--color-text)]">
                      {car
                        ? `${car.make} ${car.model} · ${car.plateNumber}`
                        : "—"}
                    </td>
                    <td className="px-4 py-3">
                      <OrderStatusBadge status={order.status} />
                    </td>
                    <td className="px-4 py-3 text-sm font-medium text-[var(--color-text)]">
                      {order.totalPrice > 0
                        ? formatPrice(order.totalPrice)
                        : "—"}
                    </td>
                    <td className="px-4 py-3 text-sm text-[var(--color-hint)]">
                      {formatDate(order.createdAt)}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>
      <p className="text-xs text-[var(--color-hint)]">
        Показано: {filtered.length} из {orders.length}
      </p>
    </div>
  );
}
