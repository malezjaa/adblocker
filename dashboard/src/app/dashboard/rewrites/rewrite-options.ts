import type {
  RewriteBehavior,
  RewriteClientCondition,
  RewriteMatchType,
  RewriteRecordType,
  RewriteTransportCondition,
} from "@/app/dashboard/settings/user-settings.ts"

export const RECORD_TYPE_OPTIONS: RewriteRecordType[] = [
  "A",
  "AAAA",
  "CNAME",
  "MX",
  "TXT",
  "PTR",
  "SRV",
  "HTTPS",
  "SVCB",
]

export const MATCH_TYPE_OPTIONS: {
  label: string
  value: RewriteMatchType
}[] = [
  { label: "Exact", value: "exact" },
  { label: "Suffix", value: "suffix" },
  { label: "Wildcard", value: "wildcard" },
  { label: "Regex", value: "regex" },
]

export const BEHAVIOR_TYPE_OPTIONS: {
  label: string
  value: RewriteBehavior["type"]
}[] = [
  { label: "Respond", value: "respond" },
  { label: "Alias", value: "alias" },
  { label: "Forward", value: "forward" },
  { label: "NXDOMAIN", value: "nxdomain" },
  { label: "No data", value: "nodata" },
]

export const TRANSPORT_OPTIONS: {
  label: string
  value: RewriteTransportCondition
}[] = [
  { label: "Plain DNS", value: "plain" },
  { label: "DoH", value: "doh" },
  { label: "DoT", value: "dot" },
  { label: "DoQ", value: "doq" },
]

export const CLIENT_ORIGIN_OPTIONS: {
  label: string
  value: RewriteClientCondition
}[] = [
  { label: "Windows", value: "windows" },
  { label: "Linux", value: "linux" },
  { label: "macOS", value: "mac" },
]

export const MATCH_VALUE_PLACEHOLDERS: Record<RewriteMatchType, string> = {
  exact: "app.example.local",
  suffix: "example.local",
  wildcard: "*.example.local",
  regex: "^(.+\\.)?example\\.local$",
}

export function matchTypeLabel(type: RewriteMatchType) {
  return (
    MATCH_TYPE_OPTIONS.find((option) => option.value === type)?.label ?? type
  )
}

export function behaviorTypeLabel(type: RewriteBehavior["type"]) {
  return (
    BEHAVIOR_TYPE_OPTIONS.find((option) => option.value === type)?.label ?? type
  )
}
