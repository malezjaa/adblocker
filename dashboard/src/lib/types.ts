export type TransportOrigin = "Plain" | "DoH" | "DoT" | "DoQ"

export type ClientOrigin = "Windows" | "Linux" | "Mac"

export type BlockOrigin =
  | { Transport: TransportOrigin }
  | { Client: { client: ClientOrigin; transport: TransportOrigin } }

export interface QueryLog {
  id: number
  domain: string
  client_ip: string
  blocked: boolean
  block_origin: BlockOrigin | null
  timestamp: number
  response_time: number
  device_id: string | null
  country_code: string | null
  company_name: string | null
  device: Device | null
  record_type: string
}

export interface QueryLogsResponse {
  total: number
  page: number
  per_page: number
  items: QueryLog[]
}

export interface QueryLogsOptions {
  page?: number
  perPage?: number
  domain?: string
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

export type TopBlocked = {
  domain: string
  hits_blocked: number
  hits_total: number
  last_seen: number
  avg_response_time: number
}

export interface HourStat {
  hour: string
  total: number
  blocked: number
}

export type StatsChange = {
  total_queries: number
  total_blocked: number
  total_allowed: number
  block_rate: number
  avg_response_time: number
}

export type Stats = {
  total_queries: number
  total_blocked: number
  total_allowed: number
  block_rate: number
  avg_response_time: number
  top_countries: CountryStat[]
  top_companies: PopularStat[]
  weekly_change: StatsChange | null
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

export type Compatibility = "Safe" | "Balanced" | "Aggressive"

export type CategoryFlag =
  | "ADS"
  | "PRIVACY"
  | "SECURITY"
  | "NSFW"
  | "GAMBLING"
  | "FAKE_NEWS"

export type Categories = string

export type List = {
  id: string
  name: string
  description: string
  homepage: string
  url: string

  categories: Categories
  compatibility: Compatibility

  recommended: boolean
  default_enabled: boolean
  priority: number

  domains?: number
  enabled?: boolean
}

export type Rule = {
  domain: string
  action: "allow" | "block"
}

export type PaginatedRules = {
  total: number
  page: number
  per_page: number
  items: Rule[]
}

export type RulesQuery = {
  page?: number
  perPage?: number
  domain?: string
}
