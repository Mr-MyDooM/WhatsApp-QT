import * as SliderPrimitive from "@radix-ui/react-slider";
import type { ComponentProps } from "react";
import { cn } from "../../lib/utils";

export function Slider({ className, ...props }: ComponentProps<typeof SliderPrimitive.Root>) {
  return (
    <SliderPrimitive.Root className={cn("relative flex h-5 w-full items-center", className)} {...props}>
      <SliderPrimitive.Track className="relative h-1 grow rounded-full bg-switch-off">
        <SliderPrimitive.Range className="absolute h-full rounded-full bg-accent" />
      </SliderPrimitive.Track>
      <SliderPrimitive.Thumb className="block size-[18px] rounded-full bg-white shadow-thumb" />
    </SliderPrimitive.Root>
  );
}
