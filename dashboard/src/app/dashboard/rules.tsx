import { useState } from "react"
import {
  ArrowLeftRight,
  Loader2,
  Pencil,
  Plus,
  Search,
  ShieldBan,
  ShieldCheck,
  Trash2,
} from "lucide-react"

import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
import type { Rule } from "@/lib/types.ts"
import {
  useCreateRule,
  useDeleteRule,
  useRules,
  useUpdateRule,
} from "@/lib/api.ts"

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
import { Kbd } from "@/components/ui/kbd.tsx"
import { toast } from "sonner"
import { useDebounce } from "@/hooks/use-debounce.ts"
import { DataPagination } from "@/components/data-pagination.tsx"

type Action = "allow" | "block"

export default function Rules() {
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(10)
  const [searchInput, setSearchInput] = useState("")
  const domainFilter = useDebounce(searchInput.trim(), 300)

  function handlePerPageChange(value: number) {
    setPerPage(value)
    setPage(1)
  }

  const { data, isLoading, error } = useRules({
    page,
    perPage,
    domain: domainFilter || undefined,
  })

  const createRule = useCreateRule()
  const updateRule = useUpdateRule()
  const deleteRule = useDeleteRule()

  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingDomain, setEditingDomain] = useState<string | null>(null)
  const [domainValue, setDomainValue] = useState("")

  const [deleteTarget, setDeleteTarget] = useState<string | null>(null)

  const isEditing = editingDomain !== null
  const isSaving = createRule.isPending || updateRule.isPending

  function openCreateDialog() {
    setEditingDomain(null)
    setDomainValue("")
    setDialogOpen(true)
  }

  function openEditDialog(rule: Rule) {
    setEditingDomain(rule.domain)
    setDomainValue(rule.domain)
    setDialogOpen(true)
  }

  function handleSave(action: Action) {
    const domain = domainValue.trim()
    if (!domain) return

    const mutation = isEditing ? updateRule : createRule
    mutation.mutate(
      { domain, action },
      {
        onSuccess: () => {
          setDialogOpen(false)
          toast.success("Rule saved", {
            description:
              "Changes can take a few minutes to apply. You may need to clear your browser cache to see them take effect.",
          })
        },
      }
    )
  }

  function handleInvert(rule: Rule) {
    updateRule.mutate(
      {
        domain: rule.domain,
        action: rule.action === "allow" ? "block" : "allow",
      },
      {
        onSuccess: () => {
          toast.success("Rule updated", {
            description:
              "Changes can take a few minutes to apply. You may need to clear your browser cache to see them take effect.",
          })
        },
      }
    )
  }

  function confirmDelete() {
    if (!deleteTarget) return
    deleteRule.mutate(deleteTarget, {
      onSuccess: () => {
        setDeleteTarget(null)
        toast.success("Rule deleted", {
          description:
            "Changes can take a few minutes to apply. You may need to clear your browser cache to see them take effect.",
        })
      },
    })
  }

  function handleSearchChange(value: string) {
    setSearchInput(value)
    setPage(1)
  }

  const totalPages = data ? Math.max(1, Math.ceil(data.total / perPage)) : 1

  return (
    <DashboardPage>
      <DashboardCard className="w-full">
        <CardHeader className="flex flex-row items-start justify-between gap-4">
          <div>
            <CardDescription>Manage your domain rules</CardDescription>
            <CardTitle>Rules</CardTitle>
          </div>
          <Button onClick={openCreateDialog} className="gap-2">
            <Plus className="size-4" />
            Add rule
          </Button>
        </CardHeader>
        <CardContent className="flex flex-col gap-4">
          <div className="relative w-full max-w-sm">
            <Search className="absolute top-1/2 left-2.5 size-4 -translate-y-1/2 text-muted-foreground" />
            <Input
              placeholder="Filter by domain..."
              value={searchInput}
              onChange={(e) => handleSearchChange(e.target.value)}
              className="pl-8"
            />
          </div>

          {error ? (
            <p className="text-sm text-destructive">
              Couldn't load rules. Try again.
            </p>
          ) : (
            <div className="overflow-hidden rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Domain</TableHead>
                    <TableHead>Action</TableHead>
                    <TableHead className="w-40 text-right">Manage</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={3} className="h-24 text-center">
                        <Loader2 className="mx-auto size-5 animate-spin text-muted-foreground" />
                      </TableCell>
                    </TableRow>
                  ) : data && data.items.length > 0 ? (
                    data.items.map((rule) => (
                      <TableRow key={rule.domain}>
                        <TableCell className="font-mono text-sm">
                          {rule.domain}
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant={
                              rule.action === "allow"
                                ? "default"
                                : "destructive"
                            }
                            className="gap-1"
                          >
                            {rule.action === "allow" ? (
                              <ShieldCheck className="size-3" />
                            ) : (
                              <ShieldBan className="size-3" />
                            )}
                            {rule.action === "allow" ? "Allow" : "Block"}
                          </Badge>
                        </TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-1">
                            <Button
                              variant="ghost"
                              size="icon"
                              title="Invert action"
                              onClick={() => handleInvert(rule)}
                              disabled={updateRule.isPending}
                            >
                              <ArrowLeftRight className="size-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              title="Edit rule"
                              onClick={() => openEditDialog(rule)}
                            >
                              <Pencil className="size-4" />
                            </Button>
                            <Button
                              variant="destructive"
                              size="icon"
                              title="Delete rule"
                              onClick={() => setDeleteTarget(rule.domain)}
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
                        colSpan={3}
                        className="h-24 text-center text-muted-foreground"
                      >
                        No rules found.
                      </TableCell>
                    </TableRow>
                  )}
                </TableBody>
              </Table>
            </div>
          )}

          {data && data.total > 0 && (
            <DataPagination
              page={page}
              perPage={perPage}
              totalItems={data.total}
              totalPages={totalPages}
              onPageChange={setPage}
              onPerPageChange={handlePerPageChange}
            />
          )}
        </CardContent>
      </DashboardCard>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>{isEditing ? "Edit rule" : "Add rule"}</DialogTitle>
            <DialogDescription>
              {isEditing
                ? "Choose a new action for this domain."
                : "Enter a domain, then choose whether to allow or block it."}
            </DialogDescription>
          </DialogHeader>

          <div className="flex flex-col gap-2 py-2">
            <Label htmlFor="domain">Domain / Rule</Label>
            <Input
              id="domain"
              placeholder="example.com"
              value={domainValue}
              onChange={(e) => setDomainValue(e.target.value)}
              disabled={isEditing}
              autoFocus
            />
            <p className="text-xs text-muted-foreground">
              For <strong>Block</strong>, enter a valid Adblock Plus / uBlock
              filter (e.g. <code>||example.com^</code>,{" "}
              <code>||example.com^$third-party</code>,{" "}
              <code>||example.com^$important</code>). For <strong>Allow</strong>
              , enter a domain name such as <code>example.com</code>; it will be
              automatically converted into an exception rule.
            </p>
          </div>

          <DialogFooter className="gap-2 sm:justify-end">
            <Button
              variant="destructive"
              onClick={() => handleSave("block")}
              disabled={!domainValue.trim() || isSaving}
              className="gap-2"
            >
              {isSaving ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <ShieldBan className="size-4" />
              )}
              Block
            </Button>
            <Button
              onClick={() => handleSave("allow")}
              disabled={!domainValue.trim() || isSaving}
              className="gap-2"
            >
              {isSaving ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <ShieldCheck className="size-4" />
              )}
              Allow
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
            <AlertDialogTitle>Delete rule?</AlertDialogTitle>
            <AlertDialogDescription>
              This will remove the rule for <Kbd>{deleteTarget}</Kbd>. This
              action can't be undone.
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>Cancel</AlertDialogCancel>
            <AlertDialogAction
              onClick={confirmDelete}
              disabled={deleteRule.isPending}
              variant={"destructive"}
            >
              {deleteRule.isPending && (
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
