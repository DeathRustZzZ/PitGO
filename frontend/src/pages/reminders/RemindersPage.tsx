import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { remindersApi, clientsApi, carsApi } from "@/shared/api/mock/store";
import type { ReminderStatus } from "@/shared/types";
import { Button } from "@/shared/ui/button";
import { Input, Select, Textarea } from "@/shared/ui/input";
import { Badge } from "@/shared/ui/badge";
import { EmptyState } from "@/shared/ui/empty-state";
import { Dialog, DialogContent, DialogClose } from "@/shared/ui/dialog";
import { formatDate } from "@/shared/lib/utils";
import { Plus, Check, X } from "lucide-react";

const schema = z.object({
  clientId: z.string().min(1, "Выберите клиента"),
  carId: z.string().optional(),
  type: z.enum(["service", "oil_change", "inspection", "custom"]),
  dueDate: z.string().min(1, "Укажите дату"),
  text: z.string().min(1, "Добавьте текст"),
});
type FormValues = z.infer<typeof schema>;

const typeLabels: Record<string, string> = {
  service: "ТО",
  oil_change: "Замена масла",
  inspection: "Техосмотр",
  custom: "Произвольное",
};

const statusLabels: Record<
  ReminderStatus,
  { label: string; variant: "default" | "success" | "destructive" }
> = {
  planned: { label: "Запланировано", variant: "default" },
  sent: { label: "Отправлено", variant: "success" },
  cancelled: { label: "Отменено", variant: "destructive" },
};

type StatusFilter = ReminderStatus | "all";

export function RemindersPage() {
  const [statusFilter, setStatusFilter] = useState<StatusFilter>("planned");
  const [createOpen, setCreateOpen] = useState(false);
  const qc = useQueryClient();

  const { data: reminders = [] } = useQuery({
    queryKey: ["reminders"],
    queryFn: remindersApi.list,
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

  const {
    register,
    handleSubmit,
    watch,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues: { type: "service" },
  });

  const selectedClientId = watch("clientId");
  const clientCars = cars.filter((c) => c.clientId === selectedClientId);

  const createMutation = useMutation({
    mutationFn: (data: FormValues) =>
      remindersApi.create({
        clientId: data.clientId,
        carId: data.carId || undefined,
        type: data.type,
        dueDate: data.dueDate,
        text: data.text,
        status: "planned",
      }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["reminders"] });
      setCreateOpen(false);
      reset();
    },
  });

  const updateMutation = useMutation({
    mutationFn: ({ id, status }: { id: string; status: ReminderStatus }) =>
      remindersApi.update(id, { status }),
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["reminders"] }),
  });

  const filtered = reminders
    .filter((r) => statusFilter === "all" || r.status === statusFilter)
    .sort(
      (a, b) => new Date(a.dueDate).getTime() - new Date(b.dueDate).getTime(),
    );

  const today = new Date().toISOString().split("T")[0];

  return (
    <div className="max-w-4xl flex flex-col gap-4">
      <div className="flex items-center gap-3">
        <div className="flex gap-1 bg-[var(--color-bg-subtle)] rounded-[var(--radius-md)] p-1">
          {(["planned", "sent", "cancelled", "all"] as StatusFilter[]).map(
            (s) => (
              <button
                key={s}
                onClick={() => setStatusFilter(s)}
                className={`px-3 py-1.5 text-sm rounded-md transition-colors ${
                  statusFilter === s
                    ? "bg-[var(--color-surface)] text-[var(--color-text)] shadow-sm font-medium"
                    : "text-[var(--color-hint)] hover:text-[var(--color-text)]"
                }`}
              >
                {s === "all" ? "Все" : statusLabels[s].label}
              </button>
            ),
          )}
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus size={16} />
          Создать напоминание
        </Button>
      </div>

      <div className="bg-[var(--color-surface)] rounded-[var(--radius-lg)] border border-[var(--color-border)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <EmptyState
            title="Нет напоминаний"
            action={
              <Button onClick={() => setCreateOpen(true)}>
                <Plus size={16} />
                Создать напоминание
              </Button>
            }
          />
        ) : (
          <div className="divide-y divide-[var(--color-border)]">
            {filtered.map((r) => {
              const client = clientMap[r.clientId];
              const car = r.carId ? carMap[r.carId] : undefined;
              const isOverdue = r.status === "planned" && r.dueDate < today;
              return (
                <div
                  key={r.id}
                  className={`flex items-start gap-4 px-5 py-4 ${isOverdue ? "bg-red-50/50" : ""}`}
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2 mb-1">
                      <Badge variant="default">{typeLabels[r.type]}</Badge>
                      <Badge variant={statusLabels[r.status].variant}>
                        {statusLabels[r.status].label}
                      </Badge>
                      {isOverdue && (
                        <Badge variant="destructive">Просрочено</Badge>
                      )}
                    </div>
                    <p className="text-sm text-[var(--color-text)]">{r.text}</p>
                    <div className="flex items-center gap-3 mt-1 text-xs text-[var(--color-hint)]">
                      <span>{client?.fullName ?? "—"}</span>
                      {car && (
                        <span>
                          · {car.make} {car.model} {car.plateNumber}
                        </span>
                      )}
                      <span>· {formatDate(r.dueDate)}</span>
                    </div>
                  </div>
                  {r.status === "planned" && (
                    <div className="flex items-center gap-1 flex-shrink-0">
                      <button
                        onClick={() =>
                          updateMutation.mutate({ id: r.id, status: "sent" })
                        }
                        className="flex items-center gap-1 px-2.5 py-1.5 text-xs text-green-700 bg-green-100 hover:bg-green-200 rounded-[var(--radius-md)] transition-colors font-medium"
                      >
                        <Check size={13} />
                        Выполнено
                      </button>
                      <button
                        onClick={() =>
                          updateMutation.mutate({
                            id: r.id,
                            status: "cancelled",
                          })
                        }
                        className="p-1.5 text-[var(--color-hint)] hover:text-red-500 hover:bg-red-50 rounded-[var(--radius-md)] transition-colors"
                      >
                        <X size={14} />
                      </button>
                    </div>
                  )}
                </div>
              );
            })}
          </div>
        )}
      </div>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent title="Новое напоминание">
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
            <Select
              id="carId"
              label="Автомобиль"
              disabled={!selectedClientId}
              {...register("carId")}
            >
              <option value="">Без автомобиля</option>
              {clientCars.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.make} {c.model} · {c.plateNumber}
                </option>
              ))}
            </Select>
            <Select id="type" label="Тип" {...register("type")}>
              <option value="service">ТО</option>
              <option value="oil_change">Замена масла</option>
              <option value="inspection">Техосмотр</option>
              <option value="custom">Произвольное</option>
            </Select>
            <Input
              id="dueDate"
              label="Дата *"
              type="date"
              error={errors.dueDate?.message}
              {...register("dueDate")}
            />
            <Textarea
              id="text"
              label="Текст напоминания *"
              error={errors.text?.message}
              rows={3}
              {...register("text")}
            />
            <div className="flex gap-2 justify-end pt-2">
              <DialogClose asChild>
                <Button type="button" variant="secondary">
                  Отмена
                </Button>
              </DialogClose>
              <Button type="submit" disabled={createMutation.isPending}>
                {createMutation.isPending ? "Создаём..." : "Создать"}
              </Button>
            </div>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  );
}
