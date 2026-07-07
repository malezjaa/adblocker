"use client"

import { CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useStats } from "@/lib/api.ts"
import { Skeleton } from "@/components/ui/skeleton.tsx"
import NumberFlow from "@number-flow/react"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import { Activity, ArrowDown, ArrowUp, Ban, Percent, Timer } from "lucide-react"

interface BadgeProps {
  value: number | undefined
  unit?: string
}

function ChangeBadge({ value, unit = "%" }: BadgeProps) {
  if (value === undefined || value === null) return null

  const positive = value >= 0
  const formatted = `${value.toFixed(1)}${unit}`

  return (
    <div className={"flex flex-row gap-3 text-sm!"}>
      <span
        className={`inline-flex items-center gap-1 rounded-full py-0.5 pr-1.5 ${
          positive
            ? "text-green-700 dark:text-green-400"
            : "text-red-700 dark:text-red-400"
        }`}
      >
        {positive ? (
          <ArrowUp className="size-3.5" />
        ) : (
          <ArrowDown className="size-3.5" />
        )}
        {formatted}
      </span>

      <p className={"text-muted-foreground"}>vs prior 7 days</p>
    </div>
  )
}

export function SectionCards() {
  const { data, isLoading } = useStats()

  return (
    <div className="grid border-collapse grid-cols-1 px-4 lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4">
      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription className="flex items-center gap-1.5">
            <Activity className="size-3.5" />
            Total DNS requests
          </CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <NumberFlow value={data?.total_queries || 0} />
            )}
          </CardTitle>
          {isLoading ? (
            <Skeleton className="h-4 w-16" />
          ) : (
            <ChangeBadge value={data?.weekly_change?.total_queries} />
          )}
        </CardHeader>
      </DashboardCard>

      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription className="flex items-center gap-1.5">
            <Ban className="size-3.5" />
            Blocked DNS requests
          </CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <NumberFlow value={data?.total_blocked || 0} />
            )}
          </CardTitle>
          {isLoading ? (
            <Skeleton className="h-4 w-16" />
          ) : (
            <ChangeBadge value={data?.weekly_change?.total_blocked} />
          )}
        </CardHeader>
      </DashboardCard>

      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription className="flex items-center gap-1.5">
            <Percent className="size-3.5" />
            Block rate
          </CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <>
                <NumberFlow value={data?.block_rate || 0} />%
              </>
            )}
          </CardTitle>
          {isLoading ? (
            <Skeleton className="h-4 w-16" />
          ) : (
            <ChangeBadge value={data?.weekly_change?.block_rate} unit=" pp" />
          )}
        </CardHeader>
      </DashboardCard>

      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription className="flex items-center gap-1.5">
            <Timer className="size-3.5" />
            Average response time
          </CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <p>
                <NumberFlow value={data?.avg_response_time || 0} />
                ms
              </p>
            )}
          </CardTitle>
          {isLoading ? (
            <Skeleton className="h-4 w-16" />
          ) : (
            <ChangeBadge value={data?.weekly_change?.avg_response_time} />
          )}
        </CardHeader>
      </DashboardCard>
    </div>
  )
}
