import { Navigate } from "react-router"
import { useAuthStatus } from "@/lib/auth.ts"
import type { ReactNode } from "react"

export function ProtectedRoute({ children }: { children: ReactNode }) {
  const { data, isLoading } = useAuthStatus()

  if (isLoading) {
    return null
  }

  if (!data || !data.logged_in) {
    return <Navigate to="/login" replace />
  }

  return <>{children}</>
}
