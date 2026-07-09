import { Loader2, Pencil, Plus, Search, Trash2 } from "lucide-react"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx"
import { Button } from "@/components/ui/button.tsx"
import { Input } from "@/components/ui/input.tsx"
import { Badge } from "@/components/ui/badge.tsx"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table.tsx"
import type { RewriteEntry } from "@/app/dashboard/settings/user-settings.ts"
import { matchTypeLabel } from "./rewrite-options.ts"
import { behaviorLabel } from "./rewrite-utils.ts"

type RewritesTableProps = {
  entries: RewriteEntry[] | undefined
  error: unknown
  isLoading: boolean
  searchInput: string
  onSearchInputChange: (value: string) => void
  onCreate: () => void
  onEdit: (entry: RewriteEntry) => void
  onDelete: (entry: RewriteEntry) => void
}

export function RewritesTable({
  entries,
  error,
  isLoading,
  searchInput,
  onSearchInputChange,
  onCreate,
  onEdit,
  onDelete,
}: RewritesTableProps) {
  return (
    <DashboardCard className="w-full">
      <CardHeader className="flex flex-row items-start justify-between gap-4">
        <div>
          <CardDescription>Manage DNS rewrites</CardDescription>
          <CardTitle>Rewrites</CardTitle>
        </div>
        <Button onClick={onCreate} className="gap-2">
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
            onChange={(event) => onSearchInputChange(event.target.value)}
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
                ) : entries && entries.length > 0 ? (
                  entries.map((entry) => (
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
                            onClick={() => onEdit(entry)}
                          >
                            <Pencil className="size-4" />
                          </Button>
                          <Button
                            variant="destructive"
                            size="icon"
                            title="Delete rewrite"
                            onClick={() => onDelete(entry)}
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
  )
}
