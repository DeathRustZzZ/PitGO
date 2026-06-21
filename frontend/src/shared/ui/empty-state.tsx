import { cn } from "@/shared/lib/utils";

type Props = {
  title: string;
  description?: string;
  action?: React.ReactNode;
  className?: string;
};

export function EmptyState({ title, description, action, className }: Props) {
  return (
    <div
      className={cn(
        "flex flex-col items-center justify-center py-16 text-center",
        className,
      )}
    >
      <div className="text-4xl mb-3">📋</div>
      <p
        className="text-base font-semibold"
        style={{ color: "var(--color-text)" }}
      >
        {title}
      </p>
      {description && (
        <p
          className="text-sm mt-1 max-w-sm"
          style={{ color: "var(--color-hint)" }}
        >
          {description}
        </p>
      )}
      {action && <div className="mt-4">{action}</div>}
    </div>
  );
}
