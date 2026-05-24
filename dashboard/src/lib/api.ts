import { useQuery } from "@tanstack/react-query"

export type Stats = {
  total_queries: number
  total_blocked: number
  total_allowed: number
  block_rate: number
  avg_response_time: number
}

const BASE_URL = "http://127.0.0.64"

export async function api<T>(url: string): Promise<T> {
  const res = await fetch(`${BASE_URL}/${url}`)

  if (!res.ok) {
    throw new Error(`HTTP ${res.status}`)
  }

  return (await res.json()) as Promise<T>
}

export const useStats = () => {
  return useQuery<Stats>({
    queryKey: ["stats"],
    queryFn: () => api<Stats>("api/stats"),
  })
}

export interface DayStat {
  day: string
  total: number
  blocked: number
}

export async function fetchChartData(days?: number): Promise<DayStat[]> {
  const params = days !== undefined ? `?days=${days}` : ""
  return api<DayStat[]>(`api/chart-data${params}`)
}

export const useChartData = (days?: number) => {
  return useQuery<DayStat[]>({
    queryKey: ["chart-data", days],
    queryFn: () => fetchChartData(days),
  })
}

export type TopBlocked = {
  domain: string,
  hits_blocked: number,
  hits_total: number,
}

export const useTopBlocked = () =>
  useQuery<TopBlocked[]>({ queryKey: ["top-blocked"], queryFn: () => api<TopBlocked[]>("api/top") });