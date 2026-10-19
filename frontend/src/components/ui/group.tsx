import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

// Grouped form, like System Settings: rounded white block, one row per setting.
export function Group({ children, title, className }: { children: ReactNode; title?: string; className?: string }) {
  return (
    <section className={cn("mb-5", className)}>
      {title && <h2 className="mb-1 px-3 text-xs font-semibold text-secondary">{title}</h2>}
      <div className="divide-y divide-separator rounded-[10px] bg-group shadow-group">{children}</div>
    </section>
  );
}

/** One setting. `error` shows under the row, next to the control that failed. */
export function Row({
  label,
  htmlFor,
  hint,
  error,
  children,
}: {
  label: string;
  htmlFor?: string;
  hint?: string;
  error?: string;
  children?: ReactNode;
}) {
  return (
    <div className="px-3 py-2">
      <div className="flex min-h-[22px] items-center justify-between gap-4">
        <div className="min-w-0">
          <label htmlFor={htmlFor} className="text-[13px] text-label">
            {label}
          </label>
          {hint && <p className="text-xs text-secondary">{hint}</p>}
        </div>
        <div className="flex shrink-0 items-center gap-2">{children}</div>
      </div>
      {error && (
        <p role="alert" className="mt-1 text-xs text-danger">
          {error}
        </p>
      )}
    </div>
  );
}
