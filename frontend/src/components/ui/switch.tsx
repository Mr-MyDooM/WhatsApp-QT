import * as SwitchPrimitive from "@radix-ui/react-switch";
import type { ComponentProps } from "react";
import { cn } from "../../lib/utils";

// macOS switch: 38 x 22 track, 18px white thumb.
export function Switch({ className, ...props }: ComponentProps<typeof SwitchPrimitive.Root>) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        "inline-flex h-[22px] w-[38px] shrink-0 items-center rounded-full p-[2px] transition-colors data-[state=checked]:bg-accent data-[state=unchecked]:bg-switch-off disabled:opacity-50",
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb className="block size-[18px] rounded-full bg-white shadow-thumb transition-transform data-[state=checked]:translate-x-4" />
    </SwitchPrimitive.Root>
  );
}
