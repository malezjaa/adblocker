import type { ReactNode } from "react"

import { cn } from "@/lib/utils.ts"
import { AppShell } from "@/components/app/app-shell.tsx"
import { ProtectedRoute } from "@/components/app/protected-route.tsx"

type DashboardPageProps = {
  children: ReactNode
  className?: string
}

export function DashboardPage({ children, className }: DashboardPageProps) {
  return (
    <ProtectedRoute>
      <AppShell>
        <div className="flex flex-1 flex-col">
          <div className="@container/main flex flex-1 flex-col gap-2">
            <div className={cn("flex flex-col py-4 md:py-6", className)}>
              {children}
            </div>
          </div>
        </div>
      </AppShell>
    </ProtectedRoute>
  )
}
