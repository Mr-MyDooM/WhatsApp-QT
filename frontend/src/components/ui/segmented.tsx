import * as RadioGroup from "@radix-ui/react-radio-group";
import type { ReactNode } from "react";
import { cn } from "../../lib/utils";

type Option = { value: string; label: string; icon?: ReactNode };

// macOS segmented control built on a radio group: arrow keys move the choice.
export function Segmented({
  value,
  onValueChange,
  options,
  label,
}: {
  value: string;
  onValueChange: (value: string) => void;
  options: Option[];
  label: string;
}) {
  return (
    <RadioGroup.Root
      value={value}
      onValueChange={onValueChange}
      aria-label={label}
      className="inline-flex gap-px rounded-lg bg-segment p-0.5"
    >
      {options.map((o) => (
        <RadioGroup.Item
          key={o.value}
          value={o.value}
          className={cn(
            "inline-flex h-[22px] items-center gap-1.5 rounded-md px-3 text-[13px] text-label transition-colors",
            "data-[state=checked]:bg-control data-[state=checked]:shadow-control",
          )}
        >
          {o.icon}
          {o.label}
        </RadioGroup.Item>
      ))}
    </RadioGroup.Root>
  );
}
