export type Stats = {
  total_queries: number
  total_blocked: number
  total_allowed: number
  block_rate: number
  avg_response_time: number
}

const BASE_URL = "http://127.0.0.64"

export const fetchStats = async (): Promise<Stats> => {
  const res = await fetch(`${BASE_URL}/api/stats`)

  if (!res.ok) {
    throw new Error(`Failed to fetch stats: ${res.status}`)
  }

  return res.json()
}
