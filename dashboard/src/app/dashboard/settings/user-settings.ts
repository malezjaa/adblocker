import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"

export interface UpstreamServer {
  name: string
  ip: string
}

export interface UserSettings {
  dnssec: boolean
  upstreams: UpstreamServer[]
}

const SETTINGS_QUERY_KEY = ["user-settings"] as const

const mockUserSettings: UserSettings = {
  upstreams: [
    { name: "cloudflare-dns.com", ip: "1.1.1.1" },
    { name: "cloudflare-dns.com", ip: "1.0.0.1" },
  ],
  dnssec: false,
}

function delay<T>(value: T, ms = 400): Promise<T> {
  return new Promise((resolve) => setTimeout(() => resolve(value), ms))
}

export function useUserSettings() {
  return useQuery({
    queryKey: SETTINGS_QUERY_KEY,
    queryFn: () => delay(mockUserSettings),
  })
}

export function useUpdateUserSettings() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (next: UserSettings) => delay(next, 600),
    onSuccess: (next) => {
      queryClient.setQueryData(SETTINGS_QUERY_KEY, next)
    },
  })
}
