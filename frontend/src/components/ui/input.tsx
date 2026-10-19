import type { ComponentProps } from "react";
import { cn } from "../../lib/utils";

export function Input({ className, type = "text", ...props }: ComponentProps<"input">) {
  return (
    <input
      type={type}
      className={cn(
        "h-6 w-full rounded-md border border-control-border bg-field px-2 text-[13px] text-label shadow-[inset_0_0.5px_1px_rgba(0,0,0,0.12)] placeholder:text-secondary",
        className,
      )}
      {...props}
    />
  );
}
