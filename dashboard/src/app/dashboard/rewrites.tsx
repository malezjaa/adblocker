import { useMemo, useState } from "react"
import { toast } from "sonner"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
import { useDebounce } from "@/hooks/use-debounce.ts"
import {
  useCreateRewrite,
  useDeleteRewrite,
  useRewrites,
  useUpdateRewrite,
} from "@/lib/api.ts"
import type {
  Rewrite,
  RewriteEntry,
} from "@/app/dashboard/settings/user-settings.ts"
import { DeleteRewriteDialog } from "./rewrites/delete-rewrite-dialog.tsx"
import { RewriteFormDialog } from "./rewrites/rewrite-form-dialog.tsx"
import { RewritesTable } from "./rewrites/rewrites-table.tsx"
import {
  behaviorLabel,
  defaultRewrite,
  normalizeBehavior,
  validateBehavior,
} from "./rewrites/rewrite-utils.ts"

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
      <RewritesTable
        entries={filtered}
        error={error}
        isLoading={isLoading}
        searchInput={searchInput}
        onSearchInputChange={setSearchInput}
        onCreate={openCreateDialog}
        onEdit={openEditDialog}
        onDelete={setDeleteTarget}
      />

      <RewriteFormDialog
        open={dialogOpen}
        isEditing={isEditing}
        isSaving={isSaving}
        draft={draft}
        formError={formError}
        setDraft={setDraft}
        onOpenChange={setDialogOpen}
        onSave={handleSave}
      />

      <DeleteRewriteDialog
        target={deleteTarget}
        isDeleting={deleteRewrite.isPending}
        onOpenChange={(open) => !open && setDeleteTarget(null)}
        onConfirm={confirmDelete}
      />
    </DashboardPage>
  )
}
