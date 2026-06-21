import { useState } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { z } from "zod/v4";
import { inventoryApi } from "@/shared/api/mock/store";
import { Button } from "@/shared/ui/button";
import { Input } from "@/shared/ui/input";
import { Badge } from "@/shared/ui/badge";
import { EmptyState } from "@/shared/ui/empty-state";
import { Dialog, DialogContent } from "@/shared/ui/dialog";
import { formatPrice } from "@/shared/lib/utils";
import { Plus, Search, Edit, Trash2 } from "lucide-react";
import type { InventoryItem } from "@/shared/types";

const schema = z.object({
  name: z.string().min(1, "Укажите название"),
  sku: z.string().min(1, "Укажите артикул"),
  quantity: z.number().min(0),
  minQuantity: z.number().min(0),
  purchasePrice: z.number().min(0),
  salePrice: z.number().min(0),
  supplierName: z.string().optional(),
});
type FormValues = z.infer<typeof schema>;

function ItemForm({
  defaultValues,
  onSubmit,
  isPending,
  onCancel,
}: {
  defaultValues?: Partial<FormValues>;
  onSubmit: (d: FormValues) => void;
  isPending: boolean;
  onCancel: () => void;
}) {
  const {
    register,
    handleSubmit,
    formState: { errors },
  } = useForm<FormValues>({
    resolver: zodResolver(schema),
    defaultValues,
  });
  return (
    <form onSubmit={handleSubmit(onSubmit)} className="flex flex-col gap-3">
      <Input
        id="name"
        label="Наименование *"
        error={errors.name?.message}
        {...register("name")}
      />
      <Input
        id="sku"
        label="Артикул (SKU) *"
        error={errors.sku?.message}
        placeholder="OIL-5W30-4"
        {...register("sku")}
      />
      <div className="grid grid-cols-2 gap-3">
        <Input
          id="quantity"
          label="Остаток"
          type="number"
          error={errors.quantity?.message}
          {...register("quantity", { valueAsNumber: true })}
        />
        <Input
          id="minQuantity"
          label="Мин. остаток"
          type="number"
          {...register("minQuantity", { valueAsNumber: true })}
        />
      </div>
      <div className="grid grid-cols-2 gap-3">
        <Input
          id="purchasePrice"
          label="Цена закупки (₽)"
          type="number"
          {...register("purchasePrice", { valueAsNumber: true })}
        />
        <Input
          id="salePrice"
          label="Цена продажи (₽)"
          type="number"
          {...register("salePrice", { valueAsNumber: true })}
        />
      </div>
      <Input
        id="supplierName"
        label="Поставщик"
        placeholder="ООО Запчасти"
        {...register("supplierName")}
      />
      <div className="flex gap-2 justify-end pt-2">
        <Button type="button" variant="secondary" onClick={onCancel}>
          Отмена
        </Button>
        <Button type="submit" disabled={isPending}>
          {isPending ? "Сохраняем..." : "Сохранить"}
        </Button>
      </div>
    </form>
  );
}

export function InventoryPage() {
  const [searchText, setSearchText] = useState("");
  const [createOpen, setCreateOpen] = useState(false);
  const [editItem, setEditItem] = useState<InventoryItem | null>(null);
  const qc = useQueryClient();

  const { data: items = [] } = useQuery({
    queryKey: ["inventory"],
    queryFn: inventoryApi.list,
  });

  const createMutation = useMutation({
    mutationFn: inventoryApi.create,
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["inventory"] });
      setCreateOpen(false);
    },
  });

  const updateMutation = useMutation({
    mutationFn: (data: FormValues) => inventoryApi.update(editItem!.id, data),
    onSuccess: () => {
      void qc.invalidateQueries({ queryKey: ["inventory"] });
      setEditItem(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: inventoryApi.delete,
    onSuccess: () => void qc.invalidateQueries({ queryKey: ["inventory"] }),
  });

  const filtered = items.filter((i) => {
    if (!searchText) return true;
    const q = searchText.toLowerCase();
    return (
      i.name.toLowerCase().includes(q) ||
      i.sku.toLowerCase().includes(q) ||
      i.supplierName?.toLowerCase().includes(q)
    );
  });

  const lowStockCount = items.filter((i) => i.quantity <= i.minQuantity).length;

  return (
    <div className="max-w-6xl flex flex-col gap-4">
      {lowStockCount > 0 && (
        <div className="flex items-center gap-2 bg-orange-50 border border-orange-200 rounded-[var(--radius-lg)] px-4 py-3">
          <span className="text-orange-600 font-medium text-sm">
            ⚠️ Низкий остаток:
          </span>
          <span className="text-sm text-orange-700">
            {lowStockCount} позиций требуют пополнения
          </span>
        </div>
      )}

      <div className="flex items-center gap-3">
        <div className="relative flex-1">
          <Search
            size={16}
            className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-hint)]"
          />
          <input
            type="text"
            placeholder="Поиск по названию, артикулу, поставщику..."
            value={searchText}
            onChange={(e) => setSearchText(e.target.value)}
            className="w-full pl-9 pr-3 py-2 text-sm rounded-[var(--radius-md)] border border-[var(--color-border)] focus:outline-none focus:ring-2 focus:shadow-[var(--focus-ring)] focus:border-[var(--color-primary)]"
          />
        </div>
        <Button onClick={() => setCreateOpen(true)}>
          <Plus size={16} />
          Добавить позицию
        </Button>
      </div>

      <div className="bg-[var(--color-surface)] rounded-[var(--radius-lg)] border border-[var(--color-border)] shadow-sm overflow-hidden">
        {filtered.length === 0 ? (
          <EmptyState
            title="Склад пуст"
            action={
              <Button onClick={() => setCreateOpen(true)}>
                <Plus size={16} />
                Добавить позицию
              </Button>
            }
          />
        ) : (
          <table className="w-full">
            <thead>
              <tr className="border-b border-[var(--color-border)] bg-[var(--color-bg)]">
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Наименование
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Артикул
                </th>
                <th className="text-right text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Остаток
                </th>
                <th className="text-right text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Закупка
                </th>
                <th className="text-right text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Продажа
                </th>
                <th className="text-left text-xs font-medium px-4 py-3 text-[var(--color-hint)]">
                  Поставщик
                </th>
                <th className="px-4 py-3" />
              </tr>
            </thead>
            <tbody className="divide-y divide-[var(--color-border)]">
              {filtered.map((item) => {
                const isLow = item.quantity <= item.minQuantity;
                return (
                  <tr
                    key={item.id}
                    className="hover:bg-[var(--color-bg)] transition-colors"
                  >
                    <td
                      className="px-4 py-3 text-sm font-medium text-[var(--color-text)]"
                      style={
                        isLow
                          ? { borderLeft: "3px solid #f97316" }
                          : { borderLeft: "3px solid transparent" }
                      }
                    >
                      {item.name}
                    </td>
                    <td className="px-4 py-3 text-xs font-mono text-[var(--color-hint)]">
                      {item.sku}
                    </td>
                    <td className="px-4 py-3 text-right">
                      <span
                        className={`text-sm font-medium ${isLow ? "text-orange-600" : "text-[var(--color-text)]"}`}
                      >
                        {item.quantity}
                      </span>
                      {isLow && (
                        <Badge variant="warning" className="ml-2">
                          Мало
                        </Badge>
                      )}
                    </td>
                    <td className="px-4 py-3 text-sm text-[var(--color-hint)] text-right">
                      {formatPrice(item.purchasePrice)}
                    </td>
                    <td className="px-4 py-3 text-sm font-medium text-[var(--color-text)] text-right">
                      {formatPrice(item.salePrice)}
                    </td>
                    <td className="px-4 py-3 text-sm text-[var(--color-hint)]">
                      {item.supplierName ?? "—"}
                    </td>
                    <td className="px-4 py-3">
                      <div className="flex items-center gap-1">
                        <button
                          onClick={() => setEditItem(item)}
                          className="p-1.5 text-[var(--color-hint)] hover:text-[var(--color-link)] hover:bg-[var(--color-bg-subtle)] rounded transition-colors"
                        >
                          <Edit size={14} />
                        </button>
                        <button
                          onClick={() => {
                            if (confirm("Удалить позицию?"))
                              deleteMutation.mutate(item.id);
                          }}
                          className="p-1.5 text-[var(--color-hint)] hover:text-red-600 hover:bg-[var(--color-bg-subtle)] rounded transition-colors"
                        >
                          <Trash2 size={14} />
                        </button>
                      </div>
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        )}
      </div>

      <Dialog open={createOpen} onOpenChange={setCreateOpen}>
        <DialogContent title="Новая позиция склада">
          <ItemForm
            defaultValues={{
              quantity: 0,
              minQuantity: 1,
              purchasePrice: 0,
              salePrice: 0,
            }}
            onSubmit={(d) => createMutation.mutate(d)}
            isPending={createMutation.isPending}
            onCancel={() => setCreateOpen(false)}
          />
        </DialogContent>
      </Dialog>

      <Dialog
        open={!!editItem}
        onOpenChange={(o) => {
          if (!o) setEditItem(null);
        }}
      >
        <DialogContent title="Редактировать позицию">
          {editItem && (
            <ItemForm
              defaultValues={editItem}
              onSubmit={(d) => updateMutation.mutate(d)}
              isPending={updateMutation.isPending}
              onCancel={() => setEditItem(null)}
            />
          )}
        </DialogContent>
      </Dialog>
    </div>
  );
}
