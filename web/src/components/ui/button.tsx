import * as React from "react"
import { Slot } from "@radix-ui/react-slot"
import { cva, type VariantProps } from "class-variance-authority"

import { cn } from "@/lib/utils"

const buttonVariants = cva(
  // Focus indicator: previously `focus-visible:ring-2 ring-ring ring-offset-2`
  // rendered a bright brand-ORANGE halo (because --ring = --brand) whenever a
  // button held keyboard focus OR matched WebKit's tap-to-focus heuristics on
  // mobile — users reported "random orange edges" on icons / ghost buttons.
  // Now we use a thin (1px) neutral foreground-tinted ring with NO offset,
  // which is visible enough for keyboard a11y but doesn't read as a brand
  // accent. Per-variant hover:bg-* + active:scale still handle pointer feedback.
  "inline-flex items-center justify-center whitespace-nowrap rounded-md text-sm font-medium transition-all duration-150 focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-foreground/30 disabled:pointer-events-none disabled:opacity-50 active:scale-[0.97]",
  {
    variants: {
      variant: {
        default: "bg-primary text-primary-foreground hover:bg-primary-hover",
        destructive:
          "bg-destructive text-destructive-foreground hover:bg-destructive-hover",
        outline:
          "border border-input bg-card hover:bg-accent hover:text-accent-foreground",
        secondary:
          "bg-secondary text-secondary-foreground hover:bg-secondary-hover",
        ghost: "hover:bg-accent hover:text-accent-foreground",
        link: "text-primary underline-offset-4 hover:underline",
      },
      size: {
        default: "h-10 px-4 py-2",
        sm: "h-9 rounded-md px-3",
        lg: "h-11 rounded-md px-8",
        icon: "h-10 w-10",
        // Extra-small: inline mini buttons (28px) — list rows, text-adjacent actions
        xs: "h-7 rounded-md px-2 text-xs",
        // Small icon button (32×32) — inside lists / cards
        "icon-sm": "h-8 w-8",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  }
)

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, style, ...props }, ref) => {
    const Comp = asChild ? Slot : "button"
    return (
      <Comp
        className={cn(buttonVariants({ variant, size, className }))}
        ref={ref}
        style={{ touchAction: 'manipulation', ...style }}
        {...props}
      />
    )
  }
)
Button.displayName = "Button"

export { Button, buttonVariants }

// ─────────────────────────────────────────────────────────────────────────────
// IconButton — convenience wrapper for icon-only buttons.
//
// Use this instead of bare <button class="p-1 rounded-md hover:bg-muted">.
// Defaults: ghost variant, muted-foreground text, hover -> foreground + muted bg.
// Sizes: "sm" (h-8 w-8, 32px) for inline/list, "md" (h-10 w-10, 40px) for
// standalone primary icon actions.
// ─────────────────────────────────────────────────────────────────────────────

export interface IconButtonProps
  extends Omit<ButtonProps, "size"> {
  size?: "sm" | "md"
}

const IconButton = React.forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ className, size = "sm", variant = "ghost", ...props }, ref) => (
    <Button
      ref={ref}
      variant={variant}
      size={size === "sm" ? "icon-sm" : "icon"}
      className={cn(
        "shrink-0 text-muted-foreground hover:text-foreground hover:bg-muted",
        className,
      )}
      {...props}
    />
  ),
)
IconButton.displayName = "IconButton"

export { IconButton }
