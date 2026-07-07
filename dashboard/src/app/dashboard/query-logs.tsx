import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
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
import React, { useState } from "react"
import { Lock, LockOpen, Search } from "lucide-react"
import { DeviceBadge } from "@/components/devices/device-badge.tsx"
import { format } from "date-fns"
import { useDebounce } from "@/hooks/use-debounce.ts"
import { countryFlag } from "@/lib/utils.ts"
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group.tsx"
import { DataPagination } from "@/components/data-pagination.tsx"

export function QueryLogs() {
  useStatsWs()
  const [page, setPage] = useState(1)
  const [perPage, setPerPage] = useState(30)
  const [domainSearch, setDomainSearch] = useState("")
  const debouncedDomain = useDebounce(domainSearch, 400)

  const { data, isLoading, error } = useQueryLogs({
    page,
    perPage,
    domain: debouncedDomain || undefined,
  })
  const totalPages = data ? Math.max(1, Math.ceil(data.total / perPage)) : 1

  function handlePerPageChange(value: number) {
    setPerPage(value)
    setPage(1)
  }

  function handleDomainSearch(e: React.ChangeEvent<HTMLInputElement>) {
    setDomainSearch(e.target.value)
    setPage(1)
  }

  return (
    <DashboardPage>
      <DashboardCard className="w-full">
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <div className="flex flex-col space-x-2">
            <CardDescription>List of all DNS queries</CardDescription>
            <CardTitle>DNS queries</CardTitle>
          </div>

          <div className="relative w-64">
            <InputGroup>
              <InputGroupInput
                placeholder="Search domain..."
                value={domainSearch}
                onChange={handleDomainSearch}
                className="pl-8"
              />
              <InputGroupAddon>
                <Search className="text-muted-foreground" />
              </InputGroupAddon>
            </InputGroup>
          </div>
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
              <AlertDescription>Failed to load query logs.</AlertDescription>
            </Alert>
          )}
          {data && (
            <>
              <div className="overflow-hidden rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>Domain</TableHead>
                      <TableHead>Type</TableHead>
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
                          <Badge variant={"secondary"}>{log.record_type}</Badge>
                        </TableCell>
                        <TableCell>
                          <Badge
                            variant={log.blocked ? "destructive" : "secondary"}
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

              <DataPagination
                page={page}
                perPage={perPage}
                totalItems={data.total}
                totalPages={totalPages}
                onPageChange={setPage}
                onPerPageChange={handlePerPageChange}
              />
            </>
          )}
        </CardContent>
      </DashboardCard>
    </DashboardPage>
  )
}
