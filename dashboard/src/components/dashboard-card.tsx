import { cn } from "@/lib/utils"
import type * as React from "react"
import { Card } from "@/components/ui/card"

export function DashboardCard({
  className,
  ...props
}: React.ComponentProps<typeof Card>) {
  return (
    <Card
      className={cn(
        "rounded-none [border-style:none] bg-background shadow-none outline outline-1 outline-border",
        className
      )}
      {...props}
    />
  )
}
