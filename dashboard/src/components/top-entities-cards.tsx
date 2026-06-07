"use client"

import {Card, CardContent, CardDescription, CardFooter, CardHeader, CardTitle,} from "@/components/ui/card"
import {useStats} from "@/lib/api.ts"
import {Skeleton} from "@/components/ui/skeleton.tsx"
import {ArrowUpRightIcon} from "lucide-react"
import * as React from "react"
import {useLayoutEffect, useRef} from "react"
import NumberFlow from "@number-flow/react";

const chartColors = {
  total: "hsl(210, 50%, 55%)",
  blocked: "hsl(25, 60%, 55%)",
}

function countryFlag(countryCode: string) {
  const code = countryCode.trim().toUpperCase()
  if (code.length !== 2) return "🏳️"
  return String.fromCodePoint(
    ...Array.from(code).map((char) => 127397 + char.charCodeAt(0))
  )
}

function useFlipAnimation(keys: string[]) {
  const prevPositions = useRef<Map<string, number>>(new Map())
  const containerRef = useRef<HTMLUListElement>(null)

  useLayoutEffect(() => {
    const container = containerRef.current
    if (!container) return

    const items = Array.from(container.children) as HTMLElement[]

    const newPositions = new Map<string, number>()
    items.forEach((el) => {
      const key = el.dataset.key
      if (key) newPositions.set(key, el.getBoundingClientRect().top)
    })

    items.forEach((el) => {
      const key = el.dataset.key
      if (!key) return
      const prev = prevPositions.current.get(key)
      const next = newPositions.get(key)
      if (prev !== undefined && next !== undefined && prev !== next) {
        const delta = prev - next
        el.style.transition = "none"
        el.style.transform = `translateY(${delta}px)`
        el.getBoundingClientRect()
        el.style.transition = "transform 400ms cubic-bezier(0.4, 0, 0.2, 1)"
        el.style.transform = "translateY(0)"
      }
    })

    prevPositions.current = newPositions
  }, [keys.join(",")])

  return containerRef
}

function TopListCard({
  title,
  description,
  linkLabel,
  href,
  children,
  isLoading,
}: {
  title: string
  description: string
  linkLabel: string
  href: string
  children: React.ReactNode
  isLoading: boolean
}) {
  return (
    <Card className="@container/card gap-3 border-border/50 shadow-sm">
      <CardHeader className="px-5 py-3">
        <CardDescription className="text-xs">{description}</CardDescription>
        <CardTitle>{title}</CardTitle>
      </CardHeader>
      <CardContent className="px-5 py-1">
        {isLoading ? (
          <div className="space-y-1.5">
            <Skeleton className="h-4 w-full" />
            <Skeleton className="h-4 w-11/12" />
            <Skeleton className="h-4 w-10/12" />
            <Skeleton className="h-4 w-9/12" />
            <Skeleton className="h-4 w-8/12" />
          </div>
        ) : (
          children
        )}
      </CardContent>
      <CardFooter className="px-5 py-2">
        <a
          href={href}
          className="inline-flex items-center gap-1 text-xs font-medium text-primary hover:underline"
        >
          {linkLabel}
          <ArrowUpRightIcon className="size-3" />
        </a>
      </CardFooter>
    </Card>
  )
}

function CountBadge({ value, color }: { value: number; color: string }) {
  return (
    <div
      className="rounded-md px-2 py-1 text-center text-xs font-medium tabular-nums"
      style={{
        backgroundColor: `color-mix(in srgb, ${color} 10%, transparent)`,
        border: `1px solid color-mix(in srgb, ${color} 15%, transparent)`,
        color,
      }}
    >
      <NumberFlow value={value} />
    </div>
  )
}

export function TopEntitiesCards() {
  const { data, isLoading } = useStats()

  const countryKeys = (data?.top_countries || []).map((c) => c.country_code)
  const companyKeys = (data?.top_companies || []).map((c) => c.label)

  const countryListRef = useFlipAnimation(countryKeys)
  const companyListRef = useFlipAnimation(companyKeys)

  return (
    <div className="grid grid-cols-1 gap-4 px-4 lg:px-6 @xl/main:grid-cols-2 dark:*:data-[slot=card]:bg-card">
      <TopListCard
        title="Top countries"
        description="Most active country codes"
        linkLabel="See all countries"
        href="/"
        isLoading={isLoading}
      >
        <div className="grid grid-cols-[minmax(0,1fr)_120px] items-end gap-2 border-b border-border/50 pb-2">
          <span className="text-[10px] font-semibold tracking-widest text-muted-foreground uppercase">
            Country
          </span>

          <div className="grid grid-cols-2 gap-1 text-center">
            <span
              className="text-[9px] font-semibold tracking-widest uppercase"
              style={{ color: chartColors.total }}
            >
              Total
            </span>
            <span
              className="text-[9px] font-semibold tracking-widest uppercase"
              style={{ color: chartColors.blocked }}
            >
              Blocked
            </span>
          </div>
        </div>
        <ul ref={countryListRef} className="divide-y divide-border/40">
          {(data?.top_countries || []).map((item, index) => (
            <li
              key={item.country_code}
              data-key={item.country_code}
              className="grid grid-cols-[minmax(0,1fr)_120px] items-center gap-2 rounded-md px-1 py-1.5 transition-colors hover:bg-muted/40"
            >
              <div className="flex min-w-0 items-center gap-2">
                <span className="text-sm leading-none" aria-hidden="true">
                  {countryFlag(item.country_code)}
                </span>
                <div className="min-w-0">
                  <p className="truncate text-xs leading-tight font-medium">
                    {item.country_code}
                  </p>
                  <p className="text-[10px] text-muted-foreground">
                    {index === 0 ? "Rank #1" : `Rank #${index + 1}`}
                  </p>
                </div>
              </div>
              <div className="grid min-w-30 grid-cols-2 gap-1">
                <CountBadge value={item.total} color={chartColors.total} />
                <CountBadge value={item.blocked} color={chartColors.blocked} />
              </div>
            </li>
          ))}
          {!data?.top_countries?.length && (
            <li className="py-2 text-xs text-muted-foreground">
              No country data yet.
            </li>
          )}
        </ul>
      </TopListCard>

      <TopListCard
        title="Top companies"
        description="Most active company names"
        linkLabel="See all companies"
        href="/"
        isLoading={isLoading}
      >
        <div className="grid grid-cols-[minmax(0,1fr)_120px] items-end gap-2 border-b border-border/50 pb-2">
          <span className="text-[10px] font-semibold tracking-widest text-muted-foreground uppercase">
            Company
          </span>

          <div className="grid grid-cols-2 gap-1 text-center">
            <span
              className="text-[9px] font-semibold tracking-widest uppercase"
              style={{ color: chartColors.total }}
            >
              Total
            </span>
            <span
              className="text-[9px] font-semibold tracking-widest uppercase"
              style={{ color: chartColors.blocked }}
            >
              Blocked
            </span>
          </div>
        </div>
        <ul ref={companyListRef} className="divide-y divide-border/40">
          {(data?.top_companies || []).map((item, index) => (
            <li
              key={item.label}
              data-key={item.label}
              className="grid grid-cols-[minmax(0,1fr)_120px] items-center gap-2 rounded-md px-1 py-1.5 transition-colors hover:bg-muted/40"
            >
              <div className="min-w-0">
                <p className="truncate text-xs leading-tight font-medium">
                  {item.label}
                </p>
                <p className="text-[10px] text-muted-foreground">
                  {index === 0 ? "Rank #1" : `Rank #${index + 1}`}
                </p>
              </div>
              <div className="grid min-w-30 grid-cols-2 gap-1">
                <CountBadge value={item.total} color={chartColors.total} />
                <CountBadge value={item.blocked} color={chartColors.blocked} />
              </div>
            </li>
          ))}
          {!data?.top_companies?.length && (
            <li className="py-2 text-xs text-muted-foreground">
              No company data yet.
            </li>
          )}
        </ul>
      </TopListCard>
    </div>
  )
}
