import { type ReactNode, useMemo, useState } from "react"
import { CircleHelp, Loader2, Pencil, Plus, Search, Trash2 } from "lucide-react"
import { toast } from "sonner"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx"
import { Button } from "@/components/ui/button.tsx"
import { Input } from "@/components/ui/input.tsx"
import { Label } from "@/components/ui/label.tsx"
import { Badge } from "@/components/ui/badge.tsx"
import { Checkbox } from "@/components/ui/checkbox.tsx"
import { Switch } from "@/components/ui/switch.tsx"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table.tsx"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog.tsx"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog.tsx"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select.tsx"
import { Kbd } from "@/components/ui/kbd.tsx"
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip.tsx"
import { useDebounce } from "@/hooks/use-debounce.ts"
import {
  useCreateRewrite,
  useDeleteRewrite,
  useRewrites,
  useUpdateRewrite,
} from "@/lib/api.ts"
import type {
  Rewrite,
  RewriteBehavior,
  RewriteEntry,
  RewriteClientCondition,
  RewriteMatchType,
  RewriteRecord,
  RewriteRecordType,
  RewriteRecordValue,
  RewriteTransportCondition,
} from "@/app/dashboard/settings/user-settings.ts"

const RECORD_TYPE_OPTIONS: RewriteRecordType[] = [
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

const MATCH_TYPE_OPTIONS: { label: string; value: RewriteMatchType }[] = [
  { label: "Exact", value: "exact" },
  { label: "Suffix", value: "suffix" },
  { label: "Wildcard", value: "wildcard" },
  { label: "Regex", value: "regex" },
]

const BEHAVIOR_TYPE_OPTIONS: {
  label: string
  value: RewriteBehavior["type"]
}[] = [
  { label: "Respond", value: "respond" },
  { label: "Alias", value: "alias" },
  { label: "Forward", value: "forward" },
  { label: "NXDOMAIN", value: "nxdomain" },
  { label: "No data", value: "nodata" },
]

const TRANSPORT_OPTIONS: {
  label: string
  value: RewriteTransportCondition
}[] = [
  { label: "Plain DNS", value: "plain" },
  { label: "DoH", value: "doh" },
  { label: "DoT", value: "dot" },
  { label: "DoQ", value: "doq" },
]

const CLIENT_ORIGIN_OPTIONS: {
  label: string
  value: RewriteClientCondition
}[] = [
  { label: "Windows", value: "windows" },
  { label: "Linux", value: "linux" },
  { label: "macOS", value: "mac" },
]

const MATCH_VALUE_PLACEHOLDERS: Record<RewriteMatchType, string> = {
  exact: "app.example.local",
  suffix: "example.local",
  wildcard: "*.example.local",
  regex: "^(.+\\.)?example\\.local$",
}

function matchTypeLabel(type: RewriteMatchType) {
  return (
    MATCH_TYPE_OPTIONS.find((option) => option.value === type)?.label ?? type
  )
}

function behaviorTypeLabel(type: RewriteBehavior["type"]) {
  return (
    BEHAVIOR_TYPE_OPTIONS.find((option) => option.value === type)?.label ?? type
  )
}

function HelpTooltip({ children }: { children: ReactNode }) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={
          <button
            type="button"
            className="inline-flex size-4 items-center justify-center text-muted-foreground transition-colors hover:text-foreground focus-visible:ring-2 focus-visible:ring-ring/30 focus-visible:outline-none"
            aria-label="More information"
          />
        }
      >
        <CircleHelp className="size-3.5" />
      </TooltipTrigger>
      <TooltipContent className="max-w-64 leading-relaxed">
        {children}
      </TooltipContent>
    </Tooltip>
  )
}

function LabelWithHelp({
  children,
  help,
  htmlFor,
}: {
  children: ReactNode
  help: ReactNode
  htmlFor?: string
}) {
  return (
    <div className="flex items-center gap-1.5">
      <Label htmlFor={htmlFor}>{children}</Label>
      <HelpTooltip>{help}</HelpTooltip>
    </div>
  )
}

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

function defaultRecord(type: RewriteRecordType = "A"): RewriteRecord {
  return {
    type,
    value: defaultRecordValue(type),
    ttl: null,
  }
}

function defaultBehavior(
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

function defaultRewrite(): Rewrite {
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

function behaviorLabel(behavior: RewriteBehavior) {
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

function entryLabel(entry: RewriteEntry) {
  return (
    entry.rewrite.name ||
    entry.rewrite.when.value ||
    `Rewrite ${entry.index + 1}`
  )
}

function inputNumber(value: number | null) {
  return value === null ? "" : String(value)
}

function optionalNumber(value: string) {
  const trimmed = value.trim()
  return trimmed ? Number(trimmed) : null
}

function csv(values: string[]) {
  return values.join(", ")
}

function fromCsv(value: string) {
  return value
    .split(",")
    .map((part) => part.trim())
    .filter(Boolean)
}

function normalizeBehavior(behavior: RewriteBehavior): RewriteBehavior {
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

function validateBehavior(behavior: RewriteBehavior) {
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

export default function Rewrites() {
  const [searchInput, setSearchInput] = useState("")
  const filter = useDebounce(searchInput.trim().toLowerCase(), 300)

  const { data, isLoading, error } = useRewrites()
  const createRewrite = useCreateRewrite()
  const updateRewrite = useUpdateRewrite()
  const deleteRewrite = useDeleteRewrite()

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingIndex, setEditingIndex] = useState<number | null>(null)
  const [draft, setDraft] = useState<Rewrite>(defaultRewrite)
  const [formError, setFormError] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<RewriteEntry | null>(null)

  const filtered = useMemo(() => {
    if (!data) return undefined
    if (!filter) return data

    return data.filter((entry) => {
      const haystack = [
        entry.rewrite.name,
        entry.rewrite.when.type,
        entry.rewrite.when.value,
        behaviorLabel(entry.rewrite.behavior),
        JSON.stringify(entry.rewrite.behavior),
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase()

      return haystack.includes(filter)
    })
  }, [data, filter])

  const isEditing = editingIndex !== null
  const isSaving = createRewrite.isPending || updateRewrite.isPending
  const behavior = draft.behavior

  function openCreateDialog() {
    setEditingIndex(null)
    setDraft(defaultRewrite())
    setFormError(null)
    setDialogOpen(true)
  }

  function openEditDialog(entry: RewriteEntry) {
    setEditingIndex(entry.index)
    setDraft(entry.rewrite)
    setFormError(null)
    setDialogOpen(true)
  }

  function updateDraft(next: Partial<Rewrite>) {
    setDraft((prev) => ({ ...prev, ...next }))
  }

  function updateMatchType(type: string | null) {
    if (!type) return

    setDraft((prev) => ({
      ...prev,
      when: { ...prev.when, type: type as RewriteMatchType },
    }))
  }

  function updateMatchValue(value: string) {
    setDraft((prev) => ({
      ...prev,
      when: { ...prev.when, value },
    }))
  }

  function updateConditionList(
    key: keyof Rewrite["conditions"],
    values: string[]
  ) {
    setDraft((prev) => ({
      ...prev,
      conditions: { ...prev.conditions, [key]: values },
    }))
  }

  function toggleQueryType(type: RewriteRecordType) {
    setDraft((prev) => {
      const current = prev.conditions.query_types
      const queryTypes = current.includes(type)
        ? current.filter((item) => item !== type)
        : [...current, type]

      return {
        ...prev,
        conditions: { ...prev.conditions, query_types: queryTypes },
      }
    })
  }

  function toggleTransport(value: RewriteTransportCondition) {
    setDraft((prev) => {
      const current = prev.conditions.transports
      const transports = current.includes(value)
        ? current.filter((item) => item !== value)
        : [...current, value]

      return {
        ...prev,
        conditions: { ...prev.conditions, transports },
      }
    })
  }

  function toggleClientOrigin(value: RewriteClientCondition) {
    setDraft((prev) => {
      const current = prev.conditions.client_origins
      const clientOrigins = current.includes(value)
        ? current.filter((item) => item !== value)
        : [...current, value]

      return {
        ...prev,
        conditions: { ...prev.conditions, client_origins: clientOrigins },
      }
    })
  }

  function updateBehavior(next: RewriteBehavior) {
    setDraft((prev) => ({ ...prev, behavior: next }))
  }

  function updateBehaviorType(type: string | null) {
    if (!type) return
    updateBehavior(defaultBehavior(type as RewriteBehavior["type"]))
  }

  function updateRespondRecords(
    updater: (records: RewriteRecord[]) => RewriteRecord[]
  ) {
    setDraft((prev) => {
      if (prev.behavior.type !== "respond") return prev

      return {
        ...prev,
        behavior: {
          ...prev.behavior,
          records: updater(prev.behavior.records),
        },
      }
    })
  }

  function updateRecord(index: number, next: RewriteRecord) {
    updateRespondRecords((records) =>
      records.map((record, currentIndex) =>
        currentIndex === index ? next : record
      )
    )
  }

  function updateRecordType(index: number, type: RewriteRecordType) {
    updateRespondRecords((records) =>
      records.map((record, currentIndex) =>
        currentIndex === index ? defaultRecord(type) : record
      )
    )
  }

  function updateRecordValue(index: number, value: RewriteRecordValue) {
    updateRespondRecords((records) =>
      records.map((record, currentIndex) =>
        currentIndex === index ? { ...record, value } : record
      )
    )
  }

  function renderRecordFields(record: RewriteRecord, index: number) {
    const value = record.value

    switch (value.type) {
      case "A":
      case "AAAA":
      case "CNAME":
      case "PTR":
        return (
          <div className="grid gap-2">
            <Label htmlFor={`rewrite-record-${index}-value`}>Value</Label>
            <Input
              id={`rewrite-record-${index}-value`}
              value={value.value}
              onChange={(event) =>
                updateRecordValue(index, {
                  ...value,
                  value: event.target.value,
                })
              }
              placeholder={value.type === "AAAA" ? "::1" : "target.local."}
            />
          </div>
        )
      case "MX":
        return (
          <div className="grid gap-3 sm:grid-cols-[1fr_120px]">
            <div className="grid gap-2">
              <Label htmlFor={`rewrite-record-${index}-exchange`}>
                Exchange
              </Label>
              <Input
                id={`rewrite-record-${index}-exchange`}
                value={value.exchange}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    exchange: event.target.value,
                  })
                }
                placeholder="mail.local."
              />
            </div>
            <div className="grid gap-2">
              <LabelWithHelp
                htmlFor={`rewrite-record-${index}-preference`}
                help="Lower MX preference values are tried first."
              >
                Preference
              </LabelWithHelp>
              <Input
                id={`rewrite-record-${index}-preference`}
                type="number"
                min={0}
                value={value.preference}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    preference: Number(event.target.value),
                  })
                }
              />
            </div>
          </div>
        )
      case "TXT":
        return (
          <div className="grid gap-2">
            <Label htmlFor={`rewrite-record-${index}-txt`}>Text strings</Label>
            <Input
              id={`rewrite-record-${index}-txt`}
              value={csv(value.value)}
              onChange={(event) =>
                updateRecordValue(index, {
                  ...value,
                  value: fromCsv(event.target.value),
                })
              }
              placeholder="v=spf1 -all, verification-token"
            />
          </div>
        )
      case "SRV":
        return (
          <div className="grid gap-3 sm:grid-cols-[1fr_repeat(3,100px)]">
            <div className="grid gap-2">
              <Label htmlFor={`rewrite-record-${index}-target`}>Target</Label>
              <Input
                id={`rewrite-record-${index}-target`}
                value={value.target}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    target: event.target.value,
                  })
                }
                placeholder="svc.local."
              />
            </div>
            <div className="grid gap-2">
              <LabelWithHelp
                htmlFor={`rewrite-record-${index}-priority`}
                help="Lower SRV priority values are preferred first."
              >
                Priority
              </LabelWithHelp>
              <Input
                id={`rewrite-record-${index}-priority`}
                type="number"
                min={0}
                value={value.priority}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    priority: Number(event.target.value),
                  })
                }
              />
            </div>
            <div className="grid gap-2">
              <LabelWithHelp
                htmlFor={`rewrite-record-${index}-weight`}
                help="Weight balances traffic between SRV records with the same priority."
              >
                Weight
              </LabelWithHelp>
              <Input
                id={`rewrite-record-${index}-weight`}
                type="number"
                min={0}
                value={value.weight}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    weight: Number(event.target.value),
                  })
                }
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor={`rewrite-record-${index}-port`}>Port</Label>
              <Input
                id={`rewrite-record-${index}-port`}
                type="number"
                min={0}
                max={65535}
                value={value.port}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    port: Number(event.target.value),
                  })
                }
              />
            </div>
          </div>
        )
      case "HTTPS":
      case "SVCB":
        return (
          <div className="grid gap-3 sm:grid-cols-[120px_1fr]">
            <div className="grid gap-2">
              <LabelWithHelp
                htmlFor={`rewrite-record-${index}-svcb-priority`}
                help="Priority 0 is alias mode; larger values describe service endpoints."
              >
                Priority
              </LabelWithHelp>
              <Input
                id={`rewrite-record-${index}-svcb-priority`}
                type="number"
                min={0}
                value={value.priority}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    priority: Number(event.target.value),
                  })
                }
              />
            </div>
            <div className="grid gap-2">
              <Label htmlFor={`rewrite-record-${index}-svcb-target`}>
                Target
              </Label>
              <Input
                id={`rewrite-record-${index}-svcb-target`}
                value={value.target}
                onChange={(event) =>
                  updateRecordValue(index, {
                    ...value,
                    target: event.target.value,
                  })
                }
                placeholder="."
              />
            </div>
          </div>
        )
    }
  }

  function handleSave() {
    const matchValue = draft.when.value.trim()
    if (!matchValue) return

    const behaviorError = validateBehavior(draft.behavior)
    if (behaviorError) {
      setFormError(behaviorError)
      return
    }

    const rewrite: Rewrite = {
      ...draft,
      name: draft.name?.trim() || null,
      when: {
        ...draft.when,
        value: matchValue,
      },
      behavior: normalizeBehavior(draft.behavior),
    }

    const options = {
      onSuccess: () => {
        setDialogOpen(false)
        toast.success(isEditing ? "Rewrite updated" : "Rewrite added")
      },
    }

    if (isEditing && editingIndex !== null) {
      updateRewrite.mutate({ index: editingIndex, rewrite }, options)
    } else {
      createRewrite.mutate(rewrite, options)
    }
  }

  function confirmDelete() {
    if (!deleteTarget) return

    deleteRewrite.mutate(deleteTarget.index, {
      onSuccess: () => {
        setDeleteTarget(null)
        toast.success("Rewrite deleted")
      },
    })
  }

  return (
    <DashboardPage>
      <DashboardCard className="w-full">
        <CardHeader className="flex flex-row items-start justify-between gap-4">
          <div>
            <CardDescription>Manage DNS rewrites</CardDescription>
            <CardTitle>Rewrites</CardTitle>
          </div>
          <Button onClick={openCreateDialog} className="gap-2">
            <Plus className="size-4" />
            Add rewrite
          </Button>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="relative w-full max-w-sm">
            <Search className="absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Filter rewrites..."
              value={searchInput}
              onChange={(event) => setSearchInput(event.target.value)}
              className="pl-8"
            />
          </div>

          {error ? (
            <p className="text-sm text-destructive">
              Couldn't load rewrites. Try again.
            </p>
          ) : (
            <div className="overflow-hidden rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Name</TableHead>
                    <TableHead>Match</TableHead>
                    <TableHead>Behavior</TableHead>
                    <TableHead className="w-32 text-right">Manage</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={4} className="h-24 text-center">
                        <Loader2 className="mx-auto size-5 animate-spin text-muted-foreground" />
                      </TableCell>
                    </TableRow>
                  ) : filtered && filtered.length > 0 ? (
                    filtered.map((entry) => (
                      <TableRow key={entry.index}>
                        <TableCell className="font-medium">
                          <div className="flex flex-wrap items-center gap-2">
                            <span>{entry.rewrite.name || "Untitled"}</span>
                            {!entry.rewrite.enabled && (
                              <Badge variant="outline" className="font-normal">
                                Disabled
                              </Badge>
                            )}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex flex-wrap items-center gap-2">
                            <Badge variant="outline">
                              {matchTypeLabel(entry.rewrite.when.type)}
                            </Badge>
                            <span className="font-mono text-sm">
                              {entry.rewrite.when.value}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex flex-wrap gap-1.5">
                            <Badge variant="outline" className="font-normal">
                              {behaviorLabel(entry.rewrite.behavior)}
                            </Badge>
                          </div>
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-1">
                            <Button
                              variant="ghost"
                              size="icon"
                              title="Edit rewrite"
                              onClick={() => openEditDialog(entry)}
                            >
                              <Pencil className="size-4" />
                            </Button>
                            <Button
                              variant="destructive"
                              size="icon"
                              title="Delete rewrite"
                              onClick={() => setDeleteTarget(entry)}
                            >
                              <Trash2 className="size-4" />
                            </Button>
                          </div>
                        </TableCell>
                      </TableRow>
                    ))
                  ) : (
                    <TableRow>
                      <TableCell
                        colSpan={4}
                        className="h-24 text-center text-muted-foreground"
                      >
                        No rewrites found.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          )}
        </CardContent>
      </DashboardCard>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-h-[calc(100vh-2rem)] overflow-y-auto sm:max-w-4xl">
          <DialogHeader>
            <DialogTitle>
              {isEditing ? "Edit rewrite" : "Add rewrite"}
            </DialogTitle>
            <DialogDescription>
              {isEditing
                ? "Update this DNS rewrite."
                : "Create a DNS rewrite rule."}
            </DialogDescription>
          </DialogHeader>

          <div className="grid gap-5 py-2">
            <section className="grid gap-4 rounded-md border p-4">
              <div className="flex flex-wrap items-center justify-between gap-3">
                <div>
                  <h3 className="text-sm font-medium">Rule</h3>
                </div>
                <div className="flex items-center gap-2">
                  <Label htmlFor="rewrite-enabled" className="text-sm">
                    Enabled
                  </Label>
                  <Switch
                    id="rewrite-enabled"
                    checked={draft.enabled}
                    onCheckedChange={(enabled) => updateDraft({ enabled })}
                  />
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-[1fr_120px_140px_140px]">
                <div className="grid gap-2">
                  <Label htmlFor="rewrite-name">Name</Label>
                  <Input
                    id="rewrite-name"
                    value={draft.name ?? ""}
                    onChange={(event) =>
                      updateDraft({ name: event.target.value })
                    }
                    placeholder="Local service"
                  />
                </div>
                <div className="grid gap-2">
                  <LabelWithHelp
                    htmlFor="rewrite-priority"
                    help="Lower numbers run first. Rules with the same priority keep their list order."
                  >
                    Priority
                  </LabelWithHelp>
                  <Input
                    id="rewrite-priority"
                    type="number"
                    value={draft.priority}
                    onChange={(event) =>
                      updateDraft({ priority: Number(event.target.value) })
                    }
                  />
                </div>
                <div className="grid gap-2">
                  <LabelWithHelp
                    htmlFor="rewrite-ttl"
                    help="Fallback cache lifetime in seconds for synthetic answers when a behavior or record does not set its own TTL."
                  >
                    Default TTL
                  </LabelWithHelp>
                  <Input
                    id="rewrite-ttl"
                    type="number"
                    min={0}
                    value={inputNumber(draft.ttl)}
                    onChange={(event) =>
                      updateDraft({ ttl: optionalNumber(event.target.value) })
                    }
                    placeholder="300"
                  />
                </div>
                <div className="flex items-end gap-2 pb-2">
                  <Switch
                    id="rewrite-continue"
                    checked={draft.continue_processing}
                    onCheckedChange={(continue_processing) =>
                      updateDraft({ continue_processing })
                    }
                  />
                  <LabelWithHelp
                    htmlFor="rewrite-continue"
                    help="When enabled, matching continues to lower-priority rules after this rule runs."
                  >
                    Continue
                  </LabelWithHelp>
                </div>
              </div>

              <div className="grid gap-3 sm:grid-cols-[180px_1fr]">
                <div className="grid gap-2">
                  <LabelWithHelp
                    htmlFor="rewrite-match-type"
                    help="Exact matches one name, suffix includes subdomains, wildcard uses * and ?, and regex uses a regular expression."
                  >
                    Match type
                  </LabelWithHelp>
                  <Select
                    value={draft.when.type}
                    onValueChange={updateMatchType}
                  >
                    <SelectTrigger id="rewrite-match-type">
                      <SelectValue>
                        {matchTypeLabel(draft.when.type)}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {MATCH_TYPE_OPTIONS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>

                <div className="grid gap-2">
                  <LabelWithHelp
                    htmlFor="rewrite-match-value"
                    help="The domain pattern tested against the requested DNS name."
                  >
                    Match value
                  </LabelWithHelp>
                  <Input
                    id="rewrite-match-value"
                    value={draft.when.value}
                    onChange={(event) => updateMatchValue(event.target.value)}
                    placeholder={MATCH_VALUE_PLACEHOLDERS[draft.when.type]}
                    autoFocus
                  />
                </div>
              </div>
            </section>

            <section className="grid gap-4 rounded-md border p-4">
              <div>
                <h3 className="text-sm font-medium">Conditions</h3>
              </div>

              <div className="grid gap-4 lg:grid-cols-2">
                <div className="grid gap-2">
                  <div className="flex items-center gap-1.5">
                    <Label>Query types</Label>
                    <HelpTooltip>
                      Limits the rule to selected DNS record types. No selection
                      means every query type.
                    </HelpTooltip>
                  </div>
                  <div className="grid grid-cols-3 gap-2 sm:grid-cols-5">
                    {RECORD_TYPE_OPTIONS.map((type) => (
                      <label
                        key={type}
                        className="flex items-center gap-2 text-sm"
                      >
                        <Checkbox
                          checked={draft.conditions.query_types.includes(type)}
                          onCheckedChange={() => toggleQueryType(type)}
                        />
                        {type}
                      </label>
                    ))}
                  </div>
                </div>

                <div className="grid gap-2">
                  <div className="flex items-center gap-1.5">
                    <Label>Transport</Label>
                    <HelpTooltip>
                      Transport choices are OR within this group. If you also
                      choose a client OS, both groups must match.
                    </HelpTooltip>
                  </div>
                  <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                    {TRANSPORT_OPTIONS.map((transport) => (
                      <label
                        key={transport.value}
                        className="flex items-center gap-2 text-sm"
                      >
                        <Checkbox
                          checked={draft.conditions.transports.includes(
                            transport.value
                          )}
                          onCheckedChange={() =>
                            toggleTransport(transport.value)
                          }
                        />
                        {transport.label}
                      </label>
                    ))}
                  </div>
                </div>
              </div>

              <div className="grid gap-2">
                <div className="flex items-center gap-1.5">
                  <Label>Client OS</Label>
                  <HelpTooltip>
                    Client OS choices are OR within this group. With Transport
                    set to DoH and Client OS set to Windows, only Windows DoH
                    requests match.
                  </HelpTooltip>
                </div>
                <div className="grid grid-cols-2 gap-2 sm:grid-cols-4">
                  {CLIENT_ORIGIN_OPTIONS.map((client) => (
                    <label
                      key={client.value}
                      className="flex items-center gap-2 text-sm"
                    >
                      <Checkbox
                        checked={draft.conditions.client_origins.includes(
                          client.value
                        )}
                        onCheckedChange={() => toggleClientOrigin(client.value)}
                      />
                      {client.label}
                    </label>
                  ))}
                </div>
              </div>

              <div className="grid gap-2">
                <LabelWithHelp
                  htmlFor="rewrite-devices"
                  help="Comma-separated device identifiers. Leave empty to match every device."
                >
                  Devices
                </LabelWithHelp>
                <Input
                  id="rewrite-devices"
                  value={csv(draft.conditions.devices)}
                  onChange={(event) =>
                    updateConditionList("devices", fromCsv(event.target.value))
                  }
                  placeholder="laptop, workstation"
                />
              </div>
            </section>

            <section className="grid gap-4 rounded-md border p-4">
              <div className="grid gap-3 sm:grid-cols-[1fr_auto] sm:items-center">
                <div className="flex items-center gap-1.5">
                  <h3 className="text-sm font-medium">Behavior</h3>
                  <HelpTooltip>
                    Respond returns local records, alias returns a CNAME,
                    forward resolves another name, and NXDOMAIN/No data return
                    synthetic empty responses.
                  </HelpTooltip>
                </div>
                <div className="w-full sm:w-52 sm:justify-self-end">
                  <Select
                    value={behavior.type}
                    onValueChange={updateBehaviorType}
                  >
                    <SelectTrigger>
                      <SelectValue>
                        {behaviorTypeLabel(behavior.type)}
                      </SelectValue>
                    </SelectTrigger>
                    <SelectContent>
                      {BEHAVIOR_TYPE_OPTIONS.map((option) => (
                        <SelectItem key={option.value} value={option.value}>
                          {option.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              {behavior.type === "respond" && (
                <div className="grid gap-4">
                  <div className="grid gap-3 sm:grid-cols-[160px_1fr]">
                    <div className="grid gap-2">
                      <LabelWithHelp
                        htmlFor="rewrite-response-ttl"
                        help="Cache lifetime in seconds for all records in this response unless a record overrides it."
                      >
                        Response TTL
                      </LabelWithHelp>
                      <Input
                        id="rewrite-response-ttl"
                        type="number"
                        min={0}
                        value={inputNumber(behavior.ttl)}
                        onChange={(event) =>
                          updateBehavior({
                            ...behavior,
                            ttl: optionalNumber(event.target.value),
                          })
                        }
                        placeholder="Default"
                      />
                    </div>
                  </div>

                  <div className="grid gap-3">
                    {behavior.records.map((record, index) => (
                      <div
                        key={`${record.type}-${index}`}
                        className="grid gap-3 rounded-md border bg-background/40 p-3"
                      >
                        <div className="flex flex-wrap items-center justify-between gap-3">
                          <div className="flex items-center gap-2">
                            <Badge variant="outline" className="font-normal">
                              Record {index + 1}
                            </Badge>
                            <Select
                              value={record.type}
                              onValueChange={(type) =>
                                updateRecordType(
                                  index,
                                  type as RewriteRecordType
                                )
                              }
                            >
                              <SelectTrigger className="w-32">
                                <SelectValue>{record.type}</SelectValue>
                              </SelectTrigger>
                              <SelectContent>
                                {RECORD_TYPE_OPTIONS.map((type) => (
                                  <SelectItem key={type} value={type}>
                                    {type}
                                  </SelectItem>
                                ))}
                              </SelectContent>
                            </Select>
                          </div>
                          <Button
                            type="button"
                            variant="ghost"
                            size="icon"
                            title="Remove record"
                            onClick={() =>
                              updateRespondRecords((records) =>
                                records.filter(
                                  (_record, currentIndex) =>
                                    currentIndex !== index
                                )
                              )
                            }
                            disabled={behavior.records.length === 1}
                          >
                            <Trash2 className="size-4" />
                          </Button>
                        </div>

                        <div className="grid gap-3 sm:grid-cols-[1fr_120px]">
                          <div>{renderRecordFields(record, index)}</div>
                          <div className="grid content-start gap-2">
                            <LabelWithHelp
                              htmlFor={`rewrite-record-${index}-ttl`}
                              help="Cache lifetime in seconds for this specific record."
                            >
                              TTL
                            </LabelWithHelp>
                            <Input
                              id={`rewrite-record-${index}-ttl`}
                              type="number"
                              min={0}
                              value={inputNumber(record.ttl)}
                              onChange={(event) =>
                                updateRecord(index, {
                                  ...record,
                                  ttl: optionalNumber(event.target.value),
                                })
                              }
                              placeholder="Default"
                            />
                          </div>
                        </div>
                      </div>
                    ))}
                  </div>

                  <Button
                    type="button"
                    variant="outline"
                    onClick={() =>
                      updateRespondRecords((records) => [
                        ...records,
                        defaultRecord(),
                      ])
                    }
                    className="w-fit gap-2"
                  >
                    <Plus className="size-4" />
                    Add record
                  </Button>
                </div>
              )}

              {behavior.type === "alias" && (
                <div className="grid gap-3 sm:grid-cols-[1fr_160px]">
                  <div className="grid gap-2">
                    <Label htmlFor="rewrite-alias-target">Target</Label>
                    <Input
                      id="rewrite-alias-target"
                      value={behavior.target}
                      onChange={(event) =>
                        updateBehavior({
                          ...behavior,
                          target: event.target.value,
                        })
                      }
                      placeholder="target.local."
                    />
                  </div>
                  <div className="grid gap-2">
                    <LabelWithHelp
                      htmlFor="rewrite-alias-ttl"
                      help="Cache lifetime in seconds for the generated CNAME answer."
                    >
                      TTL
                    </LabelWithHelp>
                    <Input
                      id="rewrite-alias-ttl"
                      type="number"
                      min={0}
                      value={inputNumber(behavior.ttl)}
                      onChange={(event) =>
                        updateBehavior({
                          ...behavior,
                          ttl: optionalNumber(event.target.value),
                        })
                      }
                      placeholder="Default"
                    />
                  </div>
                </div>
              )}

              {behavior.type === "forward" && (
                <div className="grid gap-2">
                  <Label htmlFor="rewrite-forward-target">Target</Label>
                  <Input
                    id="rewrite-forward-target"
                    value={behavior.target}
                    onChange={(event) =>
                      updateBehavior({
                        ...behavior,
                        target: event.target.value,
                      })
                    }
                    placeholder="target.local."
                  />
                </div>
              )}
            </section>

            {formError && (
              <p className="text-sm text-destructive">{formError}</p>
            )}
          </div>

          <DialogFooter className="gap-2 sm:justify-end">
            <Button
              onClick={handleSave}
              disabled={!draft.when.value.trim() || isSaving}
              className="gap-2"
            >
              {isSaving && <Loader2 className="size-4 animate-spin" />}
              {isEditing ? "Save rewrite" : "Add rewrite"}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      <AlertDialog
        open={deleteTarget !== null}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>Delete rewrite?</AlertDialogTitle>
            <AlertDialogDescription>
              This will remove{" "}
              <Kbd>{deleteTarget ? entryLabel(deleteTarget) : ""}</Kbd>. This
              action can't be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmDelete}
              disabled={deleteRewrite.isPending}
              variant="destructive"
            >
              {deleteRewrite.isPending && (
                <Loader2 className="size-4 animate-spin" />
              )}
              Delete
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </DashboardPage>
  )
}
