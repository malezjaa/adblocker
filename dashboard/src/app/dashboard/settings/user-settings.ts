import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { api, post } from "@/lib/api.ts"
import type { Rule } from "@/lib/types.ts"

export interface UpstreamServer {
  name: string
  addr: string
}

export interface ServiceConfig {
  enabled: boolean
  port: number
}

export interface ResolverConfig {
  dnssec: boolean
  upstreams: UpstreamServer[]
}

export interface FirewallConfig {
  open_ports: boolean
}

export type CertificateStrategy = "acme" | "self-signed" | "manual" | "none"
export type AcmeChallenge = "http-01" | "dns-01" | "tls-alpn-01"

export interface AcmeConfig {
  directory_url: string
  email: string | null
  challenge: AcmeChallenge
  domain: string | null
}

export interface ManualCertConfig {
  cert_path: string | null
  key_path: string | null
}

export interface CertsConfig {
  strategy: CertificateStrategy
  acme: AcmeConfig
  manual: ManualCertConfig
}

export type RewriteMatchType = "exact" | "suffix" | "wildcard" | "regex"

export interface RewriteMatch {
  type: RewriteMatchType
  value: string
}

export type RewriteRecordType =
  | "A"
  | "AAAA"
  | "CNAME"
  | "MX"
  | "TXT"
  | "PTR"
  | "SRV"
  | "HTTPS"
  | "SVCB"

export interface RewriteConditions {
  query_types: RewriteRecordType[]
  devices: string[]
  transports: RewriteTransportCondition[]
  client_origins: RewriteClientCondition[]
}

export type RewriteTransportCondition = "plain" | "doh" | "dot" | "doq"

export type RewriteClientCondition = "windows" | "linux" | "mac"

export type RewriteRecordValue =
  | { type: "A"; value: string }
  | { type: "AAAA"; value: string }
  | { type: "CNAME"; value: string }
  | { type: "MX"; exchange: string; preference: number }
  | { type: "TXT"; value: string[] }
  | { type: "PTR"; value: string }
  | {
      type: "SRV"
      priority: number
      weight: number
      port: number
      target: string
    }
  | { type: "HTTPS"; priority: number; target: string; params: string[] }
  | { type: "SVCB"; priority: number; target: string; params: string[] }

export interface RewriteRecord {
  type: RewriteRecordType
  value: RewriteRecordValue
  ttl: number | null
}

export type RewriteBehavior =
  | { type: "respond"; records: RewriteRecord[]; ttl: number | null }
  | { type: "alias"; target: string; ttl: number | null }
  | { type: "forward"; target: string }
  | { type: "nxdomain" }
  | { type: "nodata" }

export interface Rewrite {
  name: string | null
  enabled: boolean
  priority: number
  when: RewriteMatch
  conditions: RewriteConditions
  behavior: RewriteBehavior
  ttl: number | null
  continue_processing: boolean
}

export interface RewriteEntry {
  index: number
  rewrite: Rewrite
}

export interface UserSettings {
  blocklists: string[]
  rules: Rule[] | null
  doh: ServiceConfig
  dns: ServiceConfig
  dashboard: boolean
  rewrites: Rewrite[] | null
  resolver: ResolverConfig
  certs: CertsConfig
  firewall: FirewallConfig
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
