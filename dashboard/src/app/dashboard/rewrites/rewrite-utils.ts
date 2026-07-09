import type {
  Rewrite,
  RewriteBehavior,
  RewriteEntry,
  RewriteRecord,
  RewriteRecordType,
  RewriteRecordValue,
} from "@/app/dashboard/settings/user-settings.ts"
import { behaviorTypeLabel } from "./rewrite-options.ts"

function defaultRecordValue(type: RewriteRecordType): RewriteRecordValue {
  switch (type) {
    case "A":
      return { type, value: "127.0.0.1" }
    case "AAAA":
      return { type, value: "::1" }
    case "CNAME":
      return { type, value: "target.local." }
    case "MX":
      return { type, exchange: "mail.local.", preference: 10 }
    case "TXT":
      return { type, value: ["local"] }
    case "PTR":
      return { type, value: "host.local." }
    case "SRV":
      return { type, priority: 10, weight: 10, port: 443, target: "svc.local." }
    case "HTTPS":
    case "SVCB":
      return { type, priority: 1, target: ".", params: [] }
  }
}

export function defaultRecord(type: RewriteRecordType = "A"): RewriteRecord {
  return {
    type,
    value: defaultRecordValue(type),
    ttl: null,
  }
}

export function defaultBehavior(
  type: RewriteBehavior["type"] = "respond"
): RewriteBehavior {
  switch (type) {
    case "respond":
      return { type, records: [defaultRecord()], ttl: null }
    case "alias":
      return { type, target: "target.local.", ttl: null }
    case "forward":
      return { type, target: "target.local." }
    case "nxdomain":
    case "nodata":
      return { type }
  }
}

export function defaultRewrite(): Rewrite {
  return {
    name: null,
    enabled: true,
    priority: 100,
    when: {
      type: "exact",
      value: "",
    },
    conditions: {
      query_types: [],
      devices: [],
      transports: [],
      client_origins: [],
    },
    behavior: defaultBehavior(),
    ttl: null,
    continue_processing: false,
  }
}

export function behaviorLabel(behavior: RewriteBehavior) {
  switch (behavior.type) {
    case "respond":
      return behavior.records.length > 0
        ? `Respond ${behavior.records.map((record) => record.type).join(", ")}`
        : "Respond"
    case "alias":
      return `Alias ${behavior.target}`
    case "forward":
      return `Forward ${behavior.target}`
    case "nxdomain":
      return "NXDOMAIN"
    case "nodata":
      return "NODATA"
  }
}

export function entryLabel(entry: RewriteEntry) {
  return (
    entry.rewrite.name ||
    entry.rewrite.when.value ||
    `Rewrite ${entry.index + 1}`
  )
}

export function inputNumber(value: number | null) {
  return value === null ? "" : String(value)
}

export function optionalNumber(value: string) {
  const trimmed = value.trim()
  return trimmed ? Number(trimmed) : null
}

export function csv(values: string[]) {
  return values.join(", ")
}

export function fromCsv(value: string) {
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean)
}

export function normalizeBehavior(behavior: RewriteBehavior): RewriteBehavior {
  switch (behavior.type) {
    case "respond":
      return {
        ...behavior,
        records: behavior.records.map((record) => ({
          ...record,
          value: normalizeRecordValue(record.value),
        })),
      }
    case "alias":
      return { ...behavior, target: behavior.target.trim() }
    case "forward":
      return { ...behavior, target: behavior.target.trim() }
    case "nxdomain":
    case "nodata":
      return behavior
  }
}

function normalizeRecordValue(value: RewriteRecordValue): RewriteRecordValue {
  switch (value.type) {
    case "A":
    case "AAAA":
    case "CNAME":
    case "PTR":
      return { ...value, value: value.value.trim() }
    case "MX":
      return { ...value, exchange: value.exchange.trim() }
    case "TXT":
      return { ...value, value: value.value.map((part) => part.trim()) }
    case "SRV":
    case "HTTPS":
    case "SVCB":
      return { ...value, target: value.target.trim() }
  }
}

export function validateBehavior(behavior: RewriteBehavior) {
  switch (behavior.type) {
    case "respond":
      return behavior.records.length === 0
        ? "Respond behavior needs at least one record."
        : null
    case "alias":
    case "forward":
      return behavior.target.trim()
        ? null
        : `${behaviorTypeLabel(behavior.type)} behavior needs a target.`
    case "nxdomain":
    case "nodata":
      return null
  }
}
