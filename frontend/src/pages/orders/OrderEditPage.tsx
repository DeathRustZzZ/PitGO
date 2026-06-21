import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { useNavigate, useParams } from "react-router-dom";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { ordersApi, clientsApi, carsApi } from "@/shared/api/mock/store";
import { Button } from "@/shared/ui/button";
import { Textarea, Select } from "@/shared/ui/input";
import { Card, CardContent, CardHeader, CardTitle } from "@/shared/ui/card";
import { ArrowLeft } from "lucide-react";

const schema = z.object({
  clientId: z.string().min(1, "Выберите клиента"),
  carId: z.string().min(1, "Выберите автомобиль"),
  description: z.string().min(1, "Опишите проблему"),
});

type FormValues = z.infer<typeof schema>;

export function OrderEditPage() {
  const { id } = useParams<{ id: string }>();
  const navigate = useNavigate();
  const qc = useQueryClient();

  const { data: order } = useQuery({
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

  const {
    register,
    handleSubmit,
    watch,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    values: order
      ? {
          clientId: order.clientId,
          carId: order.carId,
          description: order.description,
        }
      : undefined,
  });

  const selectedClientId = watch("clientId");
  const clientCars = cars.filter((c) => c.clientId === selectedClientId);

  const mutation = useMutation({
    mutationFn: (data: FormValues) => ordersApi.update(id!, data),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["orders"] });
      navigate(`/orders/${id}`);
    },
  });

  if (!order) return <p className="text-[var(--color-hint)]">Загрузка...</p>;

  return (
    <div className="max-w-2xl">
      <div className="flex items-center gap-3 mb-5">
        <button
          onClick={() => navigate(-1)}
          className="p-2 rounded-[var(--radius-md)] hover:bg-[var(--color-surface-muted)] transition-colors"
        >
          <ArrowLeft size={18} className="text-[var(--color-hint)]" />
        </button>
        <h2 className="text-xl font-bold text-[var(--color-text)]">
          Редактировать {order.orderNumber}
        </h2>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Данные заказа</CardTitle>
        </CardHeader>
        <CardContent>
          <form
            onSubmit={handleSubmit((d) => mutation.mutate(d))}
            className="flex flex-col gap-4"
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
                  {c.fullName} · {c.phone}
                </option>
              ))}
            </Select>

            <Select
              id="carId"
              label="Автомобиль *"
              error={errors.carId?.message}
              disabled={!selectedClientId || clientCars.length === 0}
              {...register("carId")}
            >
              <option value="">Выберите автомобиль...</option>
              {clientCars.map((c) => (
                <option key={c.id} value={c.id}>
                  {c.make} {c.model} {c.year} · {c.plateNumber}
                </option>
              ))}
            </Select>

            <Textarea
              id="description"
              label="Описание проблемы *"
              error={errors.description?.message}
              rows={4}
              {...register("description")}
            />

            <div className="flex gap-3 justify-end pt-2">
              <Button
                type="button"
                variant="secondary"
                onClick={() => navigate(-1)}
              >
                Отмена
              </Button>
              <Button type="submit" disabled={mutation.isPending}>
                {mutation.isPending ? "Сохраняем..." : "Сохранить"}
              </Button>
            </div>
          </form>
        </CardContent>
      </Card>
    </div>
  );
}
