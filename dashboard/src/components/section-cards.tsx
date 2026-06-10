"use client"

import { CardDescription, CardHeader, CardTitle } from "@/components/ui/card"
import { useStats } from "@/lib/api.ts"
import { Skeleton } from "@/components/ui/skeleton.tsx"
import NumberFlow from "@number-flow/react"
import { DashboardCard } from "./dashboard-card"

export function SectionCards() {
  const { data, isLoading } = useStats()

  return (
    <div className="grid border-collapse grid-cols-1 px-4 lg:px-6 @xl/main:grid-cols-2 @5xl/main:grid-cols-4">
      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription>Total DNS requests</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <NumberFlow value={data?.total_queries || 0} />
            )}
          </CardTitle>
        </CardHeader>
      </DashboardCard>
      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription>Blocked DNS requests</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <NumberFlow value={data?.total_blocked || 0} />
            )}
          </CardTitle>
        </CardHeader>
      </DashboardCard>
      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription>Block rate</CardDescription>
          <CardTitle className="text-2xl font-semibold tabular-nums @[250px]/card:text-3xl">
            {isLoading ? (
              <Skeleton />
            ) : (
              <>
                <NumberFlow value={data?.block_rate || 0} />%
              </>
            )}
          </CardTitle>
        </CardHeader>
      </DashboardCard>
      <DashboardCard className="@container/card">
        <CardHeader>
          <CardDescription>Average resposne time</CardDescription>
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
        </CardHeader>
      </DashboardCard>
    </div>
  )
}
