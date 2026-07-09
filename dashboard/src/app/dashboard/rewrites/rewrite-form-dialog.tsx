import type { Dispatch, SetStateAction } from "react"
import { Loader2, Plus, Trash2 } from "lucide-react"
import { Badge } from "@/components/ui/badge.tsx"
import { Button } from "@/components/ui/button.tsx"
import { Checkbox } from "@/components/ui/checkbox.tsx"
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog.tsx"
import { Input } from "@/components/ui/input.tsx"
import { Label } from "@/components/ui/label.tsx"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select.tsx"
import { Switch } from "@/components/ui/switch.tsx"
import type {
  Rewrite,
  RewriteBehavior,
  RewriteClientCondition,
  RewriteMatchType,
  RewriteRecord,
  RewriteRecordType,
  RewriteRecordValue,
  RewriteTransportCondition,
} from "@/app/dashboard/settings/user-settings.ts"
import {
  BEHAVIOR_TYPE_OPTIONS,
  CLIENT_ORIGIN_OPTIONS,
  MATCH_TYPE_OPTIONS,
  MATCH_VALUE_PLACEHOLDERS,
  RECORD_TYPE_OPTIONS,
  TRANSPORT_OPTIONS,
  behaviorTypeLabel,
  matchTypeLabel,
} from "./rewrite-options.ts"
import {
  csv,
  defaultBehavior,
  defaultRecord,
  fromCsv,
  inputNumber,
  optionalNumber,
} from "./rewrite-utils.ts"
import { HelpTooltip, LabelWithHelp } from "./rewrite-help.tsx"
import { RewriteRecordFields } from "./rewrite-record-fields.tsx"

type RewriteFormDialogProps = {
  open: boolean
  isEditing: boolean
  isSaving: boolean
  draft: Rewrite
  formError: string | null
  setDraft: Dispatch<SetStateAction<Rewrite>>
  onOpenChange: (open: boolean) => void
  onSave: () => void
}

export function RewriteFormDialog({
  open,
  isEditing,
  isSaving,
  draft,
  formError,
  setDraft,
  onOpenChange,
  onSave,
}: RewriteFormDialogProps) {
  const behavior = draft.behavior

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

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
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
                <Select value={draft.when.type} onValueChange={updateMatchType}>
                  <SelectTrigger id="rewrite-match-type">
                    <SelectValue>{matchTypeLabel(draft.when.type)}</SelectValue>
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
                        onCheckedChange={() => toggleTransport(transport.value)}
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
                  Client OS choices are OR within this group. With Transport set
                  to DoH and Client OS set to Windows, only Windows DoH requests
                  match.
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
                  Respond returns local records, alias returns a CNAME, forward
                  resolves another name, and NXDOMAIN/No data return synthetic
                  empty responses.
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
                              updateRecordType(index, type as RewriteRecordType)
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
                        <div>
                          <RewriteRecordFields
                            record={record}
                            index={index}
                            onRecordValueChange={updateRecordValue}
                          />
                        </div>
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

          {formError && <p className="text-sm text-destructive">{formError}</p>}
        </div>

        <DialogFooter className="gap-2 sm:justify-end">
          <Button
            onClick={onSave}
            disabled={!draft.when.value.trim() || isSaving}
            className="gap-2"
          >
            {isSaving && <Loader2 className="size-4 animate-spin" />}
            {isEditing ? "Save rewrite" : "Add rewrite"}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  )
}
