import { api, post } from "@/lib/api.ts"
import { useQuery } from "@tanstack/react-query"

export type AuthStatus = {
  admin_exists: boolean
  logged_in: boolean
}

export type LoginResponse = {
  success: boolean
}

export async function authLogin(password: string): Promise<LoginResponse> {
  return post<LoginResponse>("api/auth/login", { password })
}

export async function authLogout(): Promise<{ success: boolean }> {
  return post<{ success: boolean }>("api/auth/logout")
}

export async function fetchAuthStatus(): Promise<AuthStatus> {
  return api<AuthStatus>("api/auth/status")
}

export const useAuthStatus = () =>
  useQuery<AuthStatus>({
    queryKey: ["auth-status"],
    queryFn: fetchAuthStatus,
  })
