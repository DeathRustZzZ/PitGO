import { useQuery } from "@tanstack/react-query";
import { Link } from "react-router-dom";
import { ordersApi } from "@/shared/api/mock/store";
import { remindersApi } from "@/shared/api/mock/store";
import { inventoryApi } from "@/shared/api/mock/store";
import { clientsApi } from "@/shared/api/mock/store";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { OrderStatusBadge } from "@/shared/ui/badge";
import { Badge } from "@/shared/ui/badge";
import { formatPrice, formatDate } from "@/shared/lib/utils";
import { ClipboardList, Users, CheckCircle, Package } from "lucide-react";
import { ScheduleWidget } from "@/features/schedule/ui/ScheduleWidget";

function StatCard({
  label,
  value,
  icon: Icon,
  color,
}: {
  label: string;
  value: string | number;
  icon: React.ElementType;
  color: string;
}) {
  return (
    <Card>
      <CardContent className="flex items-center gap-4 py-5">
        <div
          className={`w-12 h-12 rounded-[var(--radius-lg)] flex items-center justify-center ${color}`}
        >
          <Icon size={22} className="text-white" />
        </div>
        <div>
          <p className="text-2xl font-bold text-[var(--color-text)]">{value}</p>
          <p className="text-sm text-[var(--color-hint)]">{label}</p>
        </div>
      </CardContent>
    </Card>
  );
}

const reminderTypeLabels: Record<string, string> = {
  service: "ТО",
  oil_change: "Замена масла",
  inspection: "Техосмотр",
  custom: "Произвольное",
};

export function DashboardPage() {
  const todayDate = new Date();
  const today = todayDate.toDateString();

  const { data: orders = [] } = useQuery({
    queryKey: ["orders"],
    queryFn: ordersApi.list,
  });
  const { data: clients = [] } = useQuery({
    queryKey: ["clients"],
    queryFn: clientsApi.list,
  });
  const { data: inventory = [] } = useQuery({
    queryKey: ["inventory"],
    queryFn: inventoryApi.list,
  });
  const { data: reminders = [] } = useQuery({
    queryKey: ["reminders"],
    queryFn: remindersApi.list,
  });

  const ordersToday = orders.filter(
    (o) => new Date(o.createdAt).toDateString() === today,
  );
  const activeOrders = orders.filter((o) =>
    ["new", "diagnostics", "waiting_parts", "in_progress"].includes(o.status),
  );
  const readyOrders = orders.filter((o) => o.status === "ready");
  const lowStockItems = inventory.filter((i) => i.quantity <= i.minQuantity);

  const monthStart = new Date();
  monthStart.setDate(1);
  monthStart.setHours(0, 0, 0, 0);
  const completedThisMonth = orders.filter(
    (o) =>
      o.status === "completed" &&
      o.completedAt &&
      new Date(o.completedAt) >= monthStart,
  );
  const monthRevenue = completedThisMonth.reduce((s, o) => s + o.totalPrice, 0);
  const avgPrice =
    completedThisMonth.length > 0
      ? Math.round(monthRevenue / completedThisMonth.length)
      : 0;

  const recentOrders = [...orders]
    .sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    )
    .slice(0, 5);

  const upcomingReminders = reminders
    .filter((r) => r.status === "planned")
    .sort(
      (a, b) => new Date(a.dueDate).getTime() - new Date(b.dueDate).getTime(),
    )
    .slice(0, 5);

  const clientMap = Object.fromEntries(clients.map((c) => [c.id, c]));

  return (
    <div className="flex flex-col gap-6 max-w-6xl">
      {/* Статистика */}
      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard
          label="Заказов сегодня"
          value={ordersToday.length}
          icon={ClipboardList}
          color="bg-blue-500"
        />
        <StatCard
          label="Активных заказов"
          value={activeOrders.length}
          icon={ClipboardList}
          color="bg-purple-500"
        />
        <StatCard
          label="Готово к выдаче"
          value={readyOrders.length}
          icon={CheckCircle}
          color="bg-green-500"
        />
        <StatCard
          label="Клиентов всего"
          value={clients.length}
          icon={Users}
          color="bg-[var(--color-bg)]0"
        />
      </div>

      {/* Выручка */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        <Card>
          <CardContent className="py-5">
            <p className="text-sm text-[var(--color-hint)] mb-1">
              Выручка за месяц
            </p>
            <p className="text-3xl font-bold text-[var(--color-text)]">
              {formatPrice(monthRevenue)}
            </p>
            <p className="text-xs text-[var(--color-hint)] mt-1">
              {completedThisMonth.length} завершённых заказов
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="py-5">
            <p className="text-sm text-[var(--color-hint)] mb-1">Средний чек</p>
            <p className="text-3xl font-bold text-[var(--color-text)]">
              {formatPrice(avgPrice)}
            </p>
            <p className="text-xs text-[var(--color-hint)] mt-1">
              за завершённый заказ
            </p>
          </CardContent>
        </Card>
        <Card className={lowStockItems.length > 0 ? "border-orange-200" : ""}>
          <CardContent className="py-5">
            <div className="flex items-center gap-2 mb-1">
              <Package size={16} className="text-orange-500" />
              <p className="text-sm text-[var(--color-hint)]">Низкий остаток</p>
            </div>
            <p className="text-3xl font-bold text-orange-600">
              {lowStockItems.length}
            </p>
            <p className="text-xs text-[var(--color-hint)] mt-1">
              позиций на складе
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Расписание постов */}
      <Card>
        <CardHeader className="flex flex-row items-center justify-between pb-0">
          <CardTitle>Расписание постов</CardTitle>
          <span className="text-xs text-[var(--color-hint)]">
            {todayDate.toLocaleDateString("ru-RU", {
              weekday: "long",
              day: "numeric",
              month: "long",
            })}
          </span>
        </CardHeader>
        <div className="mt-3">
          <ScheduleWidget date={todayDate} />
        </div>
      </Card>

      {/* Последние заказы + напоминания */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle>Последние заказы</CardTitle>
            <Link
              to="/app/orders"
              className="text-xs text-[var(--color-link)] hover:underline"
            >
              Все заказы
            </Link>
          </CardHeader>
          <div className="divide-y divide-[var(--color-border)]">
            {recentOrders.map((order) => (
              <Link
                key={order.id}
                to={`/app/orders/${order.id}`}
                className="flex items-center justify-between px-5 py-3 hover:bg-[var(--color-bg)] transition-colors"
              >
                <div>
                  <p className="text-sm font-medium text-[var(--color-text)]">
                    {order.orderNumber}
                  </p>
                  <p className="text-xs text-[var(--color-hint)]">
                    {clientMap[order.clientId]?.fullName ?? "—"} ·{" "}
                    {formatDate(order.createdAt)}
                  </p>
                </div>
                <OrderStatusBadge status={order.status} />
              </Link>
            ))}
          </div>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between">
            <CardTitle>Ближайшие напоминания</CardTitle>
            <Link
              to="/app/reminders"
              className="text-xs text-[var(--color-link)] hover:underline"
            >
              Все напоминания
            </Link>
          </CardHeader>
          <div className="divide-y divide-[var(--color-border)]">
            {upcomingReminders.length === 0 && (
              <p className="px-5 py-8 text-sm text-[var(--color-hint)] text-center">
                Нет напоминаний
              </p>
            )}
            {upcomingReminders.map((r) => (
              <div
                key={r.id}
                className="flex items-start justify-between px-5 py-3"
              >
                <div>
                  <p className="text-sm font-medium text-[var(--color-text)]">
                    {clientMap[r.clientId]?.fullName ?? "—"}
                  </p>
                  <p className="text-xs text-[var(--color-hint)] mt-0.5">
                    {r.text}
                  </p>
                </div>
                <div className="flex flex-col items-end gap-1">
                  <Badge variant="default">{reminderTypeLabels[r.type]}</Badge>
                  <p className="text-xs text-[var(--color-hint)]">
                    {formatDate(r.dueDate)}
                  </p>
                </div>
              </div>
            ))}
          </div>
        </Card>
      </div>
    </div>
  );
}
