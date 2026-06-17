import {useMemo, useState} from "react"
import {AppShell} from "@/components/app/app-shell"
import {DashboardCard} from "@/components/dashboard-card"
import {ProtectedRoute} from "@/components/protected-route"
import {useLists, useStatsWs, useToggleList} from "@/lib/api"
import {CardContent, CardDescription, CardHeader, CardTitle,} from "@/components/ui/card"
import {Skeleton} from "@/components/ui/skeleton.tsx"
import {Alert, AlertDescription} from "@/components/ui/alert.tsx"
import {Switch} from "@/components/ui/switch.tsx"
import {Badge} from "@/components/ui/badge"
import {Input} from "@/components/ui/input"
import {Button} from "@/components/ui/button"
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {ExternalLink, Globe, ListFilter, Search, ShieldCheck, Sparkles, X,} from "lucide-react"
import type {CategoryFlag, Compatibility, List} from "@/lib/types"
import NumberFlow from "@number-flow/react"
import {cn} from "@/lib/utils"

const CATEGORY_META: Record<
  CategoryFlag,
  { label: string; className: string }
> = {
  ADS: {
    label: "Ads",
    className:
      "border-blue-200 bg-blue-50 text-blue-700 dark:border-blue-900 dark:bg-blue-950 dark:text-blue-300",
  },
  PRIVACY: {
    label: "Privacy",
    className:
      "border-violet-200 bg-violet-50 text-violet-700 dark:border-violet-900 dark:bg-violet-950 dark:text-violet-300",
  },
  SECURITY: {
    label: "Security",
    className:
      "border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-300",
  },
  NSFW: {
    label: "NSFW",
    className:
      "border-pink-200 bg-pink-50 text-pink-700 dark:border-pink-900 dark:bg-pink-950 dark:text-pink-300",
  },
  GAMBLING: {
    label: "Gambling",
    className:
      "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-300",
  },
  FAKE_NEWS: {
    label: "Fake news",
    className:
      "border-orange-200 bg-orange-50 text-orange-700 dark:border-orange-900 dark:bg-orange-950 dark:text-orange-300",
  },
}

const ALL_CATEGORIES = Object.keys(CATEGORY_META) as CategoryFlag[]

const COMPATIBILITY_META: Record<
  Compatibility,
  { label: string; className: string }
> = {
  Safe: {
    label: "Safe",
    className:
      "border-emerald-200 bg-emerald-50 text-emerald-700 dark:border-emerald-900 dark:bg-emerald-950 dark:text-emerald-300",
  },
  Balanced: {
    label: "Balanced",
    className:
      "border-amber-200 bg-amber-50 text-amber-700 dark:border-amber-900 dark:bg-amber-950 dark:text-amber-300",
  },
  Aggressive: {
    label: "Aggressive",
    className:
      "border-red-200 bg-red-50 text-red-700 dark:border-red-900 dark:bg-red-950 dark:text-red-300",
  },
}

// serde serializes bitflags as strings divided by |, e.g.: "ADS | PRIVACY"
function parseCategories(raw: string | undefined | null): CategoryFlag[] {
  if (!raw) return []
  return raw
    .split("|")
    .map((part) => part.trim())
    .filter((part): part is CategoryFlag => part in CATEGORY_META)
}

function CategoryBadge({ category }: { category: CategoryFlag }) {
  return (
      <Badge variant="outline" className="font-normal">
        {CATEGORY_META[category].label}
      </Badge>
  )
}
function CompatibilityBadge({ value }: { value: Compatibility }) {
  const meta = COMPATIBILITY_META[value]
  return (
    <Badge variant="outline" className={cn("font-normal", meta.className)}>
      <ShieldCheck className="size-3" />
      {meta.label}
    </Badge>
  )
}

export function Lists() {
  useStatsWs()
  const { data, isLoading, error } = useLists()
  const toggleList = useToggleList()

  const [query, setQuery] = useState("")
  const [activeCategories, setActiveCategories] = useState<CategoryFlag[]>([])

  function handleToggle(list: List) {
    toggleList.mutate({
      list_id: list.id,
    })
  }

  function toggleCategoryFilter(category: CategoryFlag) {
    setActiveCategories((prev) =>
      prev.includes(category)
        ? prev.filter((c) => c !== category)
        : [...prev, category]
    )
  }

  const filtered = useMemo(() => {
    if (!data) return undefined

    const q = query.trim().toLowerCase()

    return data.filter((list) => {
      const matchesQuery =
        q.length === 0 ||
        list.name.toLowerCase().includes(q) ||
        list.description?.toLowerCase().includes(q)

      if (!matchesQuery) return false

      if (activeCategories.length === 0) return true

      const listCategories = parseCategories(list.categories)
      return activeCategories.every((c) => listCategories.includes(c))
    })
  }, [data, query, activeCategories])

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
                  <div className="flex flex-col gap-3 pb-4 sm:flex-row sm:items-center">
                    <div className="relative flex-1">
                      <Search className="absolute top-1/2 left-3 size-4 -translate-y-1/2 text-muted-foreground" />
                      <Input
                        value={query}
                        onChange={(e) => setQuery(e.target.value)}
                        placeholder="Search lists by name or description"
                        className="pl-9"
                      />
                    </div>
                    <DropdownMenu>
                      <DropdownMenuTrigger>
                        <Button variant="outline" className="gap-2">
                          <ListFilter className="size-4" />
                          Categories
                          {activeCategories.length > 0 && (
                            <Badge className="ml-1 px-1.5">
                              {activeCategories.length}
                            </Badge>
                          )}
                        </Button>
                      </DropdownMenuTrigger>
                      <DropdownMenuContent align="end" className="w-48">
                        <DropdownMenuGroup>
                          <div className="flex items-center justify-between gap-2 px-2 py-1.5">
                            <DropdownMenuLabel className="p-0 text-sm font-medium">
                              Filter by category
                            </DropdownMenuLabel>
                            {activeCategories.length > 0 && (
                              <button
                                type="button"
                                onClick={(e) => {
                                  e.preventDefault()
                                  e.stopPropagation()
                                  setActiveCategories([])
                                }}
                                className="rounded-sm p-0.5 text-muted-foreground transition-colors hover:text-foreground"
                                aria-label="Clear category filters"
                              >
                                <X className="size-3.5" />
                              </button>
                            )}
                          </div>
                          <DropdownMenuSeparator />
                          {ALL_CATEGORIES.map((category) => (
                            <DropdownMenuCheckboxItem
                              key={category}
                              checked={activeCategories.includes(category)}
                              onCheckedChange={() =>
                                toggleCategoryFilter(category)
                              }
                              onSelect={(e) => e.preventDefault()}
                            >
                              {CATEGORY_META[category].label}
                            </DropdownMenuCheckboxItem>
                          ))}
                        </DropdownMenuGroup>
                      </DropdownMenuContent>
                    </DropdownMenu>
                  </div>

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
                  {filtered && filtered.length === 0 && (
                    <div className="flex flex-col items-center gap-1 py-10 text-center">
                      <p className="font-heading text-base font-medium">
                        No lists match your filters
                      </p>
                      <p className="text-sm text-muted-foreground">
                        Try a different search term or clear the category
                        filter.
                      </p>
                    </div>
                  )}
                  {filtered && filtered.length > 0 && (
                    <ul className={"flex flex-col py-4"}>
                      {filtered.map((list, index) => {
                        const enabled = list.enabled ?? false
                        const categories = parseCategories(list.categories)
                        return (
                          <li
                            className={`flex w-full flex-row items-center justify-between gap-4 py-3 ${
                              index > 0 ? "border-t" : ""
                            }`}
                            key={`list-${list.id}`}
                          >
                            <div className={"flex flex-col gap-1.5"}>
                              <div className="flex flex-row items-center gap-2">
                                <p
                                  className={
                                    "font-heading text-base font-medium"
                                  }
                                >
                                  {list.name}
                                </p>
                                <Badge>
                                  <NumberFlow value={list.domains || 0} />
                                </Badge>
                                <CompatibilityBadge
                                    value={list.compatibility}
                                />
                                {list.recommended && (
                                  <Badge
                                    variant="outline"
                                    className="gap-1 border-sky-200 bg-sky-50 font-normal text-sky-700 dark:border-sky-900 dark:bg-sky-950 dark:text-sky-300"
                                  >
                                    <Sparkles className="size-3" />
                                    Recommended
                                  </Badge>
                                )}
                              </div>
                              <p className={"text-sm text-muted-foreground"}>
                                {list.description}
                              </p>
                              <div className="flex flex-wrap items-center gap-1.5 pt-0.5">
                                {categories.map((category) => (
                                  <CategoryBadge
                                    key={category}
                                    category={category}
                                  />
                                ))}
                              </div>
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
