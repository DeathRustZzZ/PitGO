import { useState } from "react";
import { Link } from "react-router-dom";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { clientsApi } from "@/shared/api/mock/store";
import { Button } from "@/shared/ui/button";
import { Input, Textarea } from "@/shared/ui/input";
import { EmptyState } from "@/shared/ui/empty-state";
import { Dialog, DialogContent, DialogClose } from "@/shared/ui/dialog";
import { formatDate } from "@/shared/lib/utils";
import { Plus, Search, Phone } from "lucide-react";

const schema = z.object({
  fullName: z.string().min(2, "Укажите имя"),
  phone: z.string().min(10, "Укажите телефон"),
  email: z.string().email("Неверный email").or(z.literal("")).optional(),
  comment: z.string().optional(),
});

type FormValues = z.infer<typeof schema>;

export function ClientsPage() {
  const [search, setSearch] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const qc = useQueryClient();

  const { data: clients = [] } = useQuery({
    queryKey: ["clients"],
    queryFn: clientsApi.list,
  });

  const {
    register,
    handleSubmit,
    reset,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
  });

  const createMutation = useMutation({
    mutationFn: (data: FormValues) =>
      clientsApi.create({
        fullName: data.fullName,
        phone: data.phone,
        email: data.email || undefined,
        comment: data.comment || undefined,
      }),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["clients"] });
      setCreateOpen(false);
      reset();
    },
  });

  const filtered = clients
    .filter((c) => {
      if (!search) return true;
      const q = search.toLowerCase();
      return (
        c.fullName.toLowerCase().includes(q) ||
        c.phone.includes(q) ||
        c.email?.toLowerCase().includes(q)
      );
    })
    .sort(
      (a, b) =>
        new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime(),
    );

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
            placeholder="Поиск по имени, телефону, email..."
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 text-sm rounded-[var(--radius-md)] border border-[var(--color-border)] focus:outline-none focus:ring-2 focus:shadow-[var(--focus-ring)] focus:border-[var(--color-primary)]"
          />
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus size={16} />
          Добавить клиента
        </Button>
      </div>

      <div className="bg-[var(--color-surface)] rounded-[var(--radius-lg)] border border-[var(--color-border)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <EmptyState
            title="Клиенты не найдены"
            description="Добавьте первого клиента"
            action={
              <Button onClick={() => setCreateOpen(true)}>
                <Plus size={16} />
                Добавить клиента
              </Button>
            }
          />
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b border-[var(--color-border)] bg-[var(--color-bg)]">
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  ФИО
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Телефон
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Email
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Комментарий
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Добавлен
                </th>
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)]">
              {filtered.map((c) => (
                <tr
                  key={c.id}
                  className="hover:bg-[var(--color-bg)] transition-colors"
                >
                  <td className="px-4 py-3">
                    <Link
                      to={`/clients/${c.id}`}
                      className="text-sm font-medium text-[var(--color-link)] hover:underline"
                    >
                      {c.fullName}
                    </Link>
                  </td>
                  <td className="px-4 py-3">
                    <a
                      href={`tel:${c.phone}`}
                      className="flex items-center gap-1 text-sm text-[var(--color-text)] hover:text-[var(--color-link)]"
                    >
                      <Phone size={13} />
                      {c.phone}
                    </a>
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-hint)]">
                    {c.email ?? "—"}
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-hint)] max-w-[200px] truncate">
                    {c.comment ?? "—"}
                  </td>
                  <td className="px-4 py-3 text-sm text-[var(--color-hint)]">
                    {formatDate(c.createdAt)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent title="Новый клиент">
          <form
            onSubmit={handleSubmit((d) => createMutation.mutate(d))}
            className="flex flex-col gap-3"
          >
            <Input
              id="fullName"
              label="ФИО *"
              error={errors.fullName?.message}
              placeholder="Иванов Иван Иванович"
              {...register("fullName")}
            />
            <Input
              id="phone"
              label="Телефон *"
              error={errors.phone?.message}
              placeholder="+7 (900) 000-00-00"
              {...register("phone")}
            />
            <Input
              id="email"
              label="Email"
              error={errors.email?.message}
              placeholder="email@example.com"
              {...register("email")}
            />
            <Textarea
              id="comment"
              label="Комментарий"
              placeholder="Постоянный клиент, скидка..."
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
