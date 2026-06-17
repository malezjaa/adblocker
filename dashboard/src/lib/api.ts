import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect, useRef } from "react"
import type {
  Device,
  HourStat,
  QueryLogsOptions,
  QueryLogsResponse,
  Stats,
  TopBlocked,
  List,
} from "@/lib/types.ts"

const BASE_URL = "http://127.0.0.64"

const fetchWithCreds = (input: string, init?: RequestInit) => {
  return fetch(`${BASE_URL}/${input}`, {
    ...init,
    credentials: "include",
  })
}

export async function api<T>(url: string): Promise<T> {
  const res = await fetchWithCreds(url, {
    headers: {
      "Content-Type": "application/json",
    },
  })

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }

  return res.json()
}

export async function del<T>(url: string): Promise<T> {
  const res = await fetchWithCreds(url, {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
    },
  })

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }

  return res.json()
}

export async function post<T>(url: string, data?: unknown): Promise<T> {
  const res = await fetchWithCreds(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: data !== undefined ? JSON.stringify(data) : undefined,
  })

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }

  return res.json()
}

export const useStats = () => {
  return useQuery<Stats>({
    queryKey: ["stats"],
    queryFn: () => api<Stats>("api/stats"),
  })
}

export async function fetchChartData(days?: number): Promise<HourStat[]> {
  const params = days !== undefined ? `?days=${days}` : ""
  return api<HourStat[]>(`api/chart-data${params}`)
}

export const useChartData = (days?: number) => {
  return useQuery<HourStat[]>({
    queryKey: ["chart-data", days],
    queryFn: () => fetchChartData(days),
    refetchInterval: 1000 * 30,
  })
}

export const useTopBlocked = () =>
  useQuery<TopBlocked[]>({
    queryKey: ["top-blocked"],
    queryFn: () => api<TopBlocked[]>("api/top"),
  })

const WS_URL = "ws://127.0.0.64/api/ws"

export function useStatsWs() {
  const queryClient = useQueryClient()
  const debounceTimer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(() => {
    const ws = new WebSocket(WS_URL)

    ws.onmessage = async (event) => {
      try {
        const payload = JSON.parse(event.data)

        if (!payload || !payload.stats) {
          console.warn("unexpected ws payload, missing stats field", payload)
          return
        }

        queryClient.setQueryData(["stats"], payload.stats as Stats)

        if (payload.top_blocked) {
          queryClient.setQueryData(
            ["top-blocked"],
            payload.top_blocked as unknown as TopBlocked[]
          )
        }

        if (payload.hours) {
          queryClient.setQueryData(
            ["chart-data", undefined],
            payload.hours as unknown as HourStat[]
          )
        }

        if (debounceTimer.current) {
          clearTimeout(debounceTimer.current)
        }

        debounceTimer.current = setTimeout(async () => {
          await queryClient.invalidateQueries({
            queryKey: ["devices"],
            refetchType: "all",
          })
          await queryClient.invalidateQueries({
            queryKey: ["query-logs"],
            refetchType: "all",
          })
        }, 300)
      } catch (e) {
        console.error("failed to parse ws message", e)
      }
    }

    ws.onclose = () => {
      console.log("ws disconnected")
    }

    return () => {
      ws.close()
      if (debounceTimer.current) {
        clearTimeout(debounceTimer.current)
      }
    }
  }, [queryClient])
}

export const useDevices = () =>
  useQuery<Device[]>({
    queryKey: ["devices"],
    queryFn: () => api<Device[]>("api/devices"),
  })

export const useQueryLogs = (options: QueryLogsOptions = {}) => {
  const { page = 1, perPage = 50, domain } = options

  return useQuery<QueryLogsResponse>({
    queryKey: ["query-logs", page, perPage, domain],
    queryFn: () => {
      const params = new URLSearchParams({
        page: String(page),
        per_page: String(perPage),
        ...(domain ? { domain } : {}),
      })
      return api<QueryLogsResponse>(`api/query-logs?${params}`)
    },
  })
}

export const useLists = () =>
  useQuery<List[]>({
    queryKey: ["lists"],
    queryFn: () => api<List[]>("api/lists"),
  })

type ToggleListBody = {
  list_id: string
}

export const useToggleList = () => {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (body: ToggleListBody) => post<void>("api/lists/toggle", body),

    onSuccess: async () => {
      await queryClient.invalidateQueries({
        queryKey: ["lists"],
      })
    },
  })
}
