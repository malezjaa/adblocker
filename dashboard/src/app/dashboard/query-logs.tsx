import { AppShell } from "@/components/app/app-shell"
import { DashboardCard } from "@/components/dashboard-card"
import { ProtectedRoute } from "@/components/protected-route"
import { useQueryLogs, useStatsWs } from "@/lib/api"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Alert, AlertDescription } from "@/components/ui/alert"
import { Badge } from "@/components/ui/badge"
import { Skeleton } from "@/components/ui/skeleton"
import { Button } from "@/components/ui/button"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"
import { useState } from "react"
import {
  ChevronLeft,
  ChevronRight,
  ChevronsLeft,
  ChevronsRight,
  Lock,
  LockOpen,
} from "lucide-react"
import { DeviceBadge } from "@/components/devices/device-badge.tsx"
import { format } from "date-fns"

const PER_PAGE_OPTIONS = [10, 30, 50, 100]

export function countryFlag(countryCode: string) {
  const code = countryCode.trim().toUpperCase()
  if (code.length !== 2) return "🏳️"
  return String.fromCodePoint(
    ...Array.from(code).map((char) => 127397 + char.charCodeAt(0))
  )
}

export function QueryLogs() {
  useStatsWs()
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(30)
  const { data, isLoading, error } = useQueryLogs({ page, perPage })
  const totalPages = data ? Math.ceil(data.total / perPage) : 1

  function handlePerPageChange(value: string | null) {
    setPerPage(Number(value))
    setPage(1)
  }

  return (
    <ProtectedRoute>
      <AppShell>
        <div className="flex flex-1 flex-col">
          <div className="@container/main flex flex-1 flex-col gap-2">
            <div className="flex flex-col py-4 md:py-6">
              <DashboardCard className="w-full">
                <CardHeader>
                  <CardDescription>List of all DNS queries</CardDescription>
                  <CardTitle>DNS queries</CardTitle>
                </CardHeader>
                <CardContent>
                  {isLoading && (
                    <div className="space-y-2">
                      {Array.from({ length: 10 }).map((_, i) => (
                        <Skeleton key={i} className="h-12 w-full" />
                      ))}
                    </div>
                  )}
                  {error && (
                    <Alert variant="destructive">
                      <AlertDescription>
                        Failed to load query logs.
                      </AlertDescription>
                    </Alert>
                  )}
                  {data && (
                    <>
                      <div className="overflow-hidden rounded-md border">
                        <Table>
                          <TableHeader>
                            <TableRow>
                              <TableHead>Domain</TableHead>
                              <TableHead>Status</TableHead>
                              <TableHead>Device</TableHead>
                              <TableHead>Country</TableHead>
                              <TableHead>Response</TableHead>
                              <TableHead>Time</TableHead>
                            </TableRow>
                          </TableHeader>
                          <TableBody>
                            {data.items.map((log) => (
                              <TableRow key={log.id}>
                                <TableCell className="font-mono">
                                  <div className="flex items-center gap-1.5">
                                    <TooltipProvider>
                                      <Tooltip>
                                        <TooltipTrigger>
                                          {log.block_origin === "doh" ? (
                                            <Lock className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                                          ) : (
                                            <LockOpen className="h-3.5 w-3.5 shrink-0 text-muted-foreground" />
                                          )}
                                        </TooltipTrigger>
                                        <TooltipContent>
                                          {log.block_origin === "doh"
                                            ? "Query made over DNS-over-HTTPS (encrypted)"
                                            : "Query made over plain DNS (unencrypted)"}
                                        </TooltipContent>
                                      </Tooltip>
                                    </TooltipProvider>
                                    {log.domain}
                                  </div>
                                </TableCell>
                                <TableCell>
                                  <Badge
                                    variant={
                                      log.blocked ? "destructive" : "secondary"
                                    }
                                  >
                                    {log.blocked ? "Blocked" : "Allowed"}
                                  </Badge>
                                </TableCell>
                                <TableCell>
                                  {log.device ? (
                                    <DeviceBadge device={log.device} />
                                  ) : (
                                    "-"
                                  )}
                                </TableCell>
                                <TableCell>
                                  {log.country_code
                                    ? `${countryFlag(log.country_code)} ${log.country_code}`
                                    : "-"}
                                </TableCell>
                                <TableCell>{log.response_time} ms</TableCell>
                                <TableCell>
                                  {format(
                                    new Date(log.timestamp * 1000),
                                    "MMM d, yyyy HH:mm:ss"
                                  )}
                                </TableCell>
                              </TableRow>
                            ))}
                          </TableBody>
                        </Table>
                      </div>

                      <div className="flex items-center justify-between pt-4">
                        <div className="flex items-center gap-2">
                          <p className="text-sm text-muted-foreground">
                            Rows per page
                          </p>
                          <Select
                            value={String(perPage)}
                            onValueChange={handlePerPageChange}
                          >
                            <SelectTrigger className="h-8 w-[70px]">
                              <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                              {PER_PAGE_OPTIONS.map((n) => (
                                <SelectItem key={n} value={String(n)}>
                                  {n}
                                </SelectItem>
                              ))}
                            </SelectContent>
                          </Select>
                          <p className="text-sm text-muted-foreground">
                            &mdash; {data.total.toLocaleString()} total
                          </p>
                        </div>

                        <div className="flex items-center gap-2">
                          <p className="text-sm text-muted-foreground">
                            Page {page} of {totalPages}
                          </p>

                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage(1)}
                            disabled={page === 1}
                          >
                            <ChevronsLeft className="h-4 w-4" />
                          </Button>

                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage((p) => Math.max(1, p - 1))}
                            disabled={page === 1}
                          >
                            <ChevronLeft className="h-4 w-4" />
                            Previous
                          </Button>

                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() =>
                              setPage((p) => Math.min(totalPages, p + 1))
                            }
                            disabled={page === totalPages}
                          >
                            Next
                            <ChevronRight className="h-4 w-4" />
                          </Button>

                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setPage(totalPages)}
                            disabled={page === totalPages}
                          >
                            <ChevronsRight className="h-4 w-4" />
                          </Button>
                        </div>
                      </div>
                    </>
                  )}
                </CardContent>
              </DashboardCard>
            </div>
          </div>
        </div>
      </AppShell>
    </ProtectedRoute>
  )
}
