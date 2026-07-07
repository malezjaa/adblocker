"use client"

import { type CSSProperties, useMemo } from "react"
import { Bar, BarChart, Rectangle, ReferenceLine, XAxis } from "recharts"

import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  type ChartConfig,
  ChartContainer,
  ChartTooltip,
  ChartTooltipContent,
} from "@/components/ui/chart"
import { useChartData } from "@/lib/api.ts"
import { Skeleton } from "@/components/ui/skeleton.tsx"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"

const chartConfig = {
  total: {
    label: "Total",
    color: "var(--color-chart-2)",
  },
  blocked: {
    label: "Blocked",
    color: "var(--color-chart-4)",
  },
} satisfies ChartConfig

function getRefLines(maxVal: number): number[] {
  if (maxVal <= 0) return []

  return [0.25, 0.5, 0.75, 1]
    .map((p) => Math.round(maxVal * p))
    .filter((v) => v > 0)
}

function formatLabel(val: number): string {
  if (val >= 1_000_000) return `${(val / 1_000_000).toFixed(1)}M`
  if (val >= 1_000) return `${(val / 1_000).toFixed(1)}k`
  return val.toString()
}

export function RequestsChart() {
  const { data, isLoading } = useChartData()

  const refLines = useMemo(() => {
    if (!data) return []
    const maxVal = Math.max(
      ...data.map((d) => (d.total ?? 0) + (d.blocked ?? 0))
    )

    return getRefLines(maxVal)
  }, [data])

  if (!data || isLoading) {
    return <Skeleton />
  }

  return (
    <DashboardCard className="w-full">
      <CardHeader>
        <CardDescription>Hourly DNS requests for today</CardDescription>
        <CardTitle>Total Requests</CardTitle>
      </CardHeader>
      <CardContent>
        {refLines.length === 0 ? (
          <p className="py-2 text-center text-lg text-muted-foreground">
            No chart data yet.
          </p>
        ) : (
          <ChartContainer config={chartConfig} className="h-40 w-full">
            <BarChart
              accessibilityLayer
              data={data}
              margin={{ top: 8, right: 12, bottom: 12, left: 40 }}
              barCategoryGap="20%"
            >
              {refLines.map((val) => (
                <ReferenceLine
                  key={val}
                  y={val}
                  stroke="var(--border)"
                  strokeDasharray="3 3"
                  strokeOpacity={0.6}
                  strokeWidth={1}
                  position={"start"}
                  label={{
                    value: formatLabel(val),
                    position: "left",
                    fill: "var(--muted-foreground)",
                    fontSize: 10,
                    dx: 4,
                  }}
                />
              ))}
              <defs>
                <pattern
                  id="diagonal-stripe-total"
                  patternUnits="userSpaceOnUse"
                  width="6"
                  height="6"
                >
                  <rect
                    width="6"
                    height="6"
                    fill="var(--color-total)"
                    opacity="0.15"
                  />
                  <path
                    d="M0,6 L6,0 M3,9 L9,3 M-3,3 L3,-3"
                    stroke="var(--color-total)"
                    strokeWidth="1.5"
                    opacity="0.7"
                  />
                </pattern>

                <pattern
                  id="diagonal-stripe-blocked"
                  patternUnits="userSpaceOnUse"
                  width="6"
                  height="6"
                >
                  <rect
                    width="6"
                    height="6"
                    fill="var(--color-blocked)"
                    opacity="0.15"
                  />
                  <path
                    d="M0,6 L6,0 M3,9 L9,3 M-3,3 L3,-3"
                    stroke="var(--color-blocked)"
                    strokeWidth="1.5"
                    opacity="0.7"
                  />
                </pattern>
              </defs>

              <XAxis
                dataKey="hour"
                tickLine={false}
                axisLine={false}
                tickMargin={8}
                interval={3}
                tickFormatter={(val: string) => val.replace(":00", "")}
              />
              <ChartTooltip
                cursor={false}
                content={
                  <ChartTooltipContent
                    indicator="dot"
                    className="min-w-40 gap-2.5"
                    labelFormatter={(value) => (
                      <div className="mb-0.5 flex flex-col gap-0.5 border-b border-border/50 pb-2">
                        <span className="text-xs font-medium">{value}</span>
                      </div>
                    )}
                    formatter={(value, name) => (
                      <div className="flex w-full items-center justify-between gap-2">
                        <div className="flex items-center gap-1.5">
                          <div
                            className="h-2.5 w-2.5 shrink-0 rounded-xs bg-(--color-bg)"
                            style={
                              {
                                "--color-bg": `var(--color-${name})`,
                              } as CSSProperties
                            }
                          />
                          <span className="text-muted-foreground">
                            {chartConfig[name as keyof typeof chartConfig]
                              ?.label || name}
                          </span>
                        </div>
                        <span className="font-semibold text-foreground">
                          {(value as number).toLocaleString()}
                        </span>
                      </div>
                    )}
                  />
                }
              />
              <Bar
                dataKey="blocked"
                stackId="a"
                fill="url(#diagonal-stripe-blocked)"
                stroke="var(--color-blocked)"
                strokeWidth={1}
                radius={[0, 0, 4, 4]}
                minPointSize={0}
                shape={(props: any) => {
                  if (props.value === 0) return <g />
                  return <Rectangle {...props} />
                }}
              />
              <Bar
                dataKey="total"
                stackId="a"
                fill="url(#diagonal-stripe-total)"
                stroke="var(--color-total)"
                strokeWidth={1}
                radius={[4, 4, 0, 0]}
                minPointSize={3}
              />
            </BarChart>
          </ChartContainer>
        )}
      </CardContent>
    </DashboardCard>
  )
}
