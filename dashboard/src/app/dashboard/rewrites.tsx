import { useMemo, useState } from "react"
import { Loader2, Pencil, Plus, Search, Trash2 } from "lucide-react"
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
import { Textarea } from "@/components/ui/textarea.tsx"
import { Kbd } from "@/components/ui/kbd.tsx"
import { useDebounce } from "@/hooks/use-debounce.ts"
import {
  useCreateRewrite,
  useDeleteRewrite,
  useRewrites,
  useUpdateRewrite,
} from "@/lib/api.ts"
import type {
  Rewrite,
  RewriteAction,
  RewriteEntry,
  RewriteMatchType,
} from "@/app/dashboard/settings/user-settings.ts"

const DEFAULT_ACTIONS: RewriteAction[] = [{ type: "A", value: "127.0.0.1" }]

const MATCH_TYPE_OPTIONS: { label: string; value: RewriteMatchType }[] = [
  { label: "Exact", value: "exact" },
  { label: "Regex", value: "regex" },
]

function matchTypeLabel(type: RewriteMatchType) {
  return (
    MATCH_TYPE_OPTIONS.find((option) => option.value === type)?.label ?? type
  )
}

function formatActions(actions: RewriteAction[]) {
  return JSON.stringify(actions, null, 2)
}

function defaultRewrite(): Rewrite {
  return {
    name: null,
    when: {
      type: "exact",
      value: "",
    },
    actions: DEFAULT_ACTIONS,
  }
}

function actionLabel(action: RewriteAction) {
  switch (action.type) {
    case "MX":
      return `MX ${action.preference} ${action.exchange}`
    case "SRV":
      return `SRV ${action.target}:${action.port}`
    case "TXT":
      return "TXT"
    case "NXDOMAIN":
    case "NOERROR":
      return action.type
    default:
      return `${action.type} ${"value" in action ? action.value : ""}`.trim()
  }
}

function entryLabel(entry: RewriteEntry) {
  return (
    entry.rewrite.name ||
    entry.rewrite.when.value ||
    `Rewrite ${entry.index + 1}`
  )
}

function parseActions(value: string) {
  const parsed = JSON.parse(value)

  if (
    !Array.isArray(parsed) ||
    parsed.length === 0 ||
    parsed.some(
      (action) =>
        !action || typeof action !== "object" || typeof action.type !== "string"
    )
  ) {
    throw new Error("Actions must be a non-empty array.")
  }

  return parsed as RewriteAction[]
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
  const [actionsText, setActionsText] = useState(formatActions(DEFAULT_ACTIONS))
  const [actionsError, setActionsError] = useState<string | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<RewriteEntry | null>(null)

  const filtered = useMemo(() => {
    if (!data) return undefined
    if (!filter) return data

    return data.filter((entry) => {
      const haystack = [
        entry.rewrite.name,
        entry.rewrite.when.type,
        entry.rewrite.when.value,
        formatActions(entry.rewrite.actions),
      ]
        .filter(Boolean)
        .join(" ")
        .toLowerCase()

      return haystack.includes(filter)
    })
  }, [data, filter])

  const isEditing = editingIndex !== null
  const isSaving = createRewrite.isPending || updateRewrite.isPending

  function openCreateDialog() {
    const next = defaultRewrite()
    setEditingIndex(null)
    setDraft(next)
    setActionsText(formatActions(next.actions))
    setActionsError(null)
    setDialogOpen(true)
  }

  function openEditDialog(entry: RewriteEntry) {
    setEditingIndex(entry.index)
    setDraft(entry.rewrite)
    setActionsText(formatActions(entry.rewrite.actions))
    setActionsError(null)
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

  function handleSave() {
    const matchValue = draft.when.value.trim()
    if (!matchValue) return

    let actions: RewriteAction[]
    try {
      actions = parseActions(actionsText)
      setActionsError(null)
    } catch {
      setActionsError("Actions must be valid JSON.")
      return
    }

    const rewrite: Rewrite = {
      ...draft,
      name: draft.name?.trim() || null,
      when: {
        ...draft.when,
        value: matchValue,
      },
      actions,
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
                    <TableHead>Actions</TableHead>
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
                          {entry.rewrite.name || "Untitled"}
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
                            {entry.rewrite.actions
                              .slice(0, 3)
                              .map((action, index) => (
                                <Badge
                                  key={`${entry.index}-action-${index}`}
                                  variant="outline"
                                  className="font-normal"
                                >
                                  {actionLabel(action)}
                                </Badge>
                              ))}
                            {entry.rewrite.actions.length > 3 && (
                              <Badge variant="outline" className="font-normal">
                                +{entry.rewrite.actions.length - 3}
                              </Badge>
                            )}
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
        <DialogContent>
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

          <div className="grid gap-4 py-2">
            <div className="grid gap-2">
              <Label htmlFor="rewrite-name">Name</Label>
              <Input
                id="rewrite-name"
                value={draft.name ?? ""}
                onChange={(event) => updateDraft({ name: event.target.value })}
                placeholder="Local service"
              />
            </div>

            <div className="grid gap-2 sm:grid-cols-[160px_1fr]">
              <div className="grid gap-2">
                <Label htmlFor="rewrite-match-type">Match type</Label>
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
                <Label htmlFor="rewrite-match-value">Match value</Label>
                <Input
                  id="rewrite-match-value"
                  value={draft.when.value}
                  onChange={(event) => updateMatchValue(event.target.value)}
                  placeholder="example.local"
                  autoFocus
                />
              </div>
            </div>

            <div className="grid gap-2">
              <Label htmlFor="rewrite-actions">Actions</Label>
              <Textarea
                id="rewrite-actions"
                value={actionsText}
                onChange={(event) => {
                  setActionsText(event.target.value)
                  setActionsError(null)
                }}
                spellCheck={false}
                rows={8}
                className="font-mono text-xs"
              />
              {actionsError && (
                <p className="text-sm text-destructive">{actionsError}</p>
              )}
            </div>
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
