import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { api, post } from "@/lib/api.ts"

export interface UpstreamServer {
  name: string
  addr: string
}

export interface UserSettings {
  dnssec: boolean
  upstreams: UpstreamServer[]
}

const SETTINGS_QUERY_KEY = ["user-settings"] as const

export function useUserSettings() {
  return useQuery({
    queryKey: SETTINGS_QUERY_KEY,
    queryFn: () => api<UserSettings>("api/settings"),
  })
}

export function useUpdateUserSettings() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (next: UserSettings) => post("api/settings", next),
    onSuccess: (next) => {
      queryClient.setQueryData(SETTINGS_QUERY_KEY, next)
    },
  })
}
