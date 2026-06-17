import { AppShell } from "@/components/app/app-shell"
import { DashboardCard } from "@/components/dashboard-card"
import { ProtectedRoute } from "@/components/protected-route"
import { useLists, useStatsWs, useToggleList } from "@/lib/api"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import { Skeleton } from "@/components/ui/skeleton.tsx"
import { Alert, AlertDescription } from "@/components/ui/alert.tsx"
import { Switch } from "@/components/ui/switch.tsx"
import { Badge } from "@/components/ui/badge"
import { Globe, ExternalLink } from "lucide-react"
import type { List } from "@/lib/types"

export function Lists() {
  useStatsWs()
  const { data, isLoading, error } = useLists()
  const toggleList = useToggleList()

  function handleToggle(list: List) {
    toggleList.mutate({
      list_id: list.id,
    })
  }

  return (
    <ProtectedRoute>
      <AppShell>
        <div className="flex flex-1 flex-col">
          <div className="@container/main flex flex-1 flex-col gap-2">
            <div className="flex flex-col py-4 md:py-6">
              <DashboardCard className="w-full">
                <CardHeader>
                  <CardDescription>Manage block lists</CardDescription>
                  <CardTitle>Block lists</CardTitle>
                </CardHeader>
                <CardContent>
                  {isLoading && (
                    <div className="space-y-2">
                      {Array.from({ length: 10 }).map((_, i) => (
                        <Skeleton
                          key={`lists-skeleton-${i}`}
                          className="h-12 w-full"
                        />
                      ))}
                    </div>
                  )}
                  {error && (
                    <Alert variant="destructive">
                      <AlertDescription>Failed to load lists.</AlertDescription>
                    </Alert>
                  )}
                  {data && (
                    <ul className={"flex flex-col py-4"}>
                      {data.map((list, index) => {
                        const enabled = list.enabled ?? false
                        return (
                          <li
                            className={`flex w-full flex-row items-center justify-between gap-4 py-3 ${
                              index > 0 ? "border-t" : ""
                            }`}
                            key={`list-${list.id}`}
                          >
                            <div className={"flex flex-col gap-1"}>
                              <div className="flex flex-row items-center gap-2">
                                <p
                                  className={
                                    "font-heading text-base font-medium"
                                  }
                                >
                                  {list.name}
                                </p>
                                <Badge>{list.domains || "unknown"}</Badge>
                              </div>
                              <p className={"text-sm text-muted-foreground"}>
                                {list.description}
                              </p>
                              <div className="flex flex-row items-center gap-3 pt-1">
                                {list.homepage && (
                                  <a
                                    href={list.homepage}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
                                  >
                                    <Globe className="size-3.5" />
                                    Homepage
                                  </a>
                                )}
                                {list.url && (
                                  <a
                                    href={list.url}
                                    target="_blank"
                                    rel="noopener noreferrer"
                                    className="inline-flex items-center gap-1 text-sm text-muted-foreground transition-colors hover:text-foreground"
                                  >
                                    <ExternalLink className="size-3.5" />
                                    Source
                                  </a>
                                )}
                              </div>
                            </div>
                            <Switch
                              checked={enabled}
                              disabled={toggleList.isPending}
                              onCheckedChange={() => handleToggle(list)}
                            />
                          </li>
                        )
                      })}
                    </ul>
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
