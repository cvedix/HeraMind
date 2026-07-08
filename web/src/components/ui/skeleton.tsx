import { cn } from "@/lib/utils"

function Skeleton({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={cn(
        "rounded-md",
        "bg-gradient-to-r from-muted via-muted-foreground/10 to-muted",
        "bg-[length:2000px_100%]",
        "animate-shimmer",
        className
      )}
      {...props}
    />
  )
}

export { Skeleton }
