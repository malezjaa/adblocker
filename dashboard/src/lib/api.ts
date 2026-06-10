import { useQuery, useQueryClient } from "@tanstack/react-query"
import { useEffect } from "react"

export type Stats = {
  total_queries: number
  total_blocked: number
  total_allowed: number
  block_rate: number
  avg_response_time: number
  top_countries: CountryStat[]
  top_companies: PopularStat[]
}

export type CountryStat = {
  country_code: string
  total: number
  blocked: number
}

export type PopularStat = {
  label: string
  total: number
  blocked: number
}

const BASE_URL = "http://127.0.0.64"

export async function api<T>(url: string): Promise<T> {
  const res = await fetch(`${BASE_URL}/${url}`)

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }

  return (await res.json()) as Promise<T>
}

export async function del<T>(url: string): Promise<T> {
  const res = await fetch(`${BASE_URL}/${url}`, {
    method: "DELETE",
    headers: {
      "Content-Type": "application/json",
    },
  })

  return (await res.json()) as Promise<T>
}

export async function post<T>(url: string, data: unknown): Promise<T> {
  const res = await fetch(`${BASE_URL}/${url}`, {
    method: "POST",
    body: JSON.stringify(data),
    headers: {
      "Content-Type": "application/json",
    },
  })

  return (await res.json()) as Promise<T>
}

export const useStats = () => {
  return useQuery<Stats>({
    queryKey: ["stats"],
    queryFn: () => api<Stats>("api/stats"),
  })
}

export interface HourStat {
  hour: string
  total: number
  blocked: number
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

export type TopBlocked = {
  domain: string
  hits_blocked: number
  hits_total: number
  last_seen: number
  avg_response_time: number
}

export const useTopBlocked = () =>
  useQuery<TopBlocked[]>({
    queryKey: ["top-blocked"],
    queryFn: () => api<TopBlocked[]>("api/top"),
  })

const WS_URL = "ws://127.0.0.64/api/ws"

export function useStatsWs() {
  const queryClient = useQueryClient()

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

        await queryClient.invalidateQueries({
          queryKey: ["devices"],
          refetchType: "all",
        })
      } catch (e) {
        console.error("failed to parse ws message", e)
      }
    }

    ws.onclose = () => {
      console.log("ws disconnected")
    }

    return () => {
      ws.close()
    }
  }, [queryClient])
}

export const DeviceTypes = {
  Windows: "windows",
  Linux: "linux",
  MacOs: "macos",
  Android: "android",
  iOS: "ios",
  Router: "router",
  Other: "other",
} as const

export type DeviceType = (typeof DeviceTypes)[keyof typeof DeviceTypes]

export type Device = {
  id: string
  name: string
  device_type: DeviceType
  last_seen: number
}

export const useDevices = () =>
  useQuery<Device[]>({
    queryKey: ["devices"],
    queryFn: () => api<Device[]>("api/devices"),
  })
