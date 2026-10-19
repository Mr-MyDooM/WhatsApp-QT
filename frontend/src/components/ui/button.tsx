import { cva, type VariantProps } from "class-variance-authority";
import type { ButtonHTMLAttributes } from "react";
import { cn } from "../../lib/utils";

// macOS push button: 22px high, 6px radius, hairline border. `primary` is the default button.
const buttonVariants = cva(
  "inline-flex h-[22px] items-center justify-center gap-1.5 rounded-md px-3 text-[13px] whitespace-nowrap transition-colors disabled:opacity-50",
  {
    variants: {
      variant: {
        default: "bg-control text-label shadow-control border border-control-border active:brightness-95",
        primary: "bg-accent text-accent-fg shadow-control active:brightness-90 font-medium",
      },
    },
    defaultVariants: { variant: "default" },
  },
);

type ButtonProps = ButtonHTMLAttributes<HTMLButtonElement> & VariantProps<typeof buttonVariants>;

export function Button({ className, variant, type = "button", ...props }: ButtonProps) {
  return <button type={type} className={cn(buttonVariants({ variant }), className)} {...props} />;
}
