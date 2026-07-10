import {
  ComposableMap,
  Geographies,
  Geography,
  Marker,
  ZoomableGroup,
} from "react-simple-maps"
import React, { useRef, useState } from "react"
import worldTopology from "@/assets/countries-110m.json"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import { useStats } from "@/lib/api.ts"
import { COUNTRY_COORDS } from "@/lib/country-coordinates.ts"

type TooltipState = {
  name: string
  info: string
  x: number
  y: number
} | null

function getCountryName(code: string): string {
  try {
    const displayNames = new Intl.DisplayNames(["en"], { type: "region" })
    return displayNames.of(code) ?? code
  } catch {
    return code
  }
}

function WorldMap() {
  const { data, isLoading } = useStats()
  const [tooltip, setTooltip] = useState<TooltipState>(null)
  const containerRef = useRef<HTMLDivElement>(null)
  const [position, setPosition] = useState({
    coordinates: [0, 20] as [number, number],
    zoom: 1,
  })
  const pins = (data?.top_countries ?? [])
    .map((c) => {
      const countryCode = c.country_code.trim().toUpperCase()
      const coords = COUNTRY_COORDS[countryCode]

      if (!coords) {
        return null
      }

      return {
        code: countryCode,
        name: getCountryName(countryCode),
        coords,
        total: c.total,
        blocked: c.blocked,
      }
    })
    .filter((pin): pin is NonNullable<typeof pin> => pin !== null)

  const showTooltip = (
    event: React.MouseEvent<SVGCircleElement>,
    pin: (typeof pins)[number]
  ) => {
    const rect = containerRef.current?.getBoundingClientRect()
    if (!rect) return
    setTooltip({
      name: pin.name,
      info: `${pin.total.toLocaleString()} queries · ${pin.blocked.toLocaleString()} blocked`,
      x: event.clientX - rect.left + 12,
      y: event.clientY - rect.top - 12,
    })
  }

  const maxTotal = Math.max(...pins.map((p) => p.total), 1)
  const handleMoveEnd = (position: {
    coordinates: [number, number]
    zoom: number
  }) => {
    setPosition(position)
  }

  return (
    <div ref={containerRef} className="relative w-full">
      {tooltip && (
        <div
          className="absolute z-10 border px-3 py-2 shadow-lg"
          style={{
            left: tooltip.x,
            top: tooltip.y,
            transform: "translateY(-100%)",
            pointerEvents: "none",
            background: "#141414",
            borderColor: "#262626",
          }}
        >
          <div
            style={{ color: "#e5e5e5", fontSize: "0.875rem", fontWeight: 500 }}
          >
            {tooltip.name}
          </div>
          <div style={{ color: "#a3a3a3", fontSize: "0.75rem" }}>
            {tooltip.info}
          </div>
        </div>
      )}

      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center text-sm text-muted-foreground">
          Loading…
        </div>
      )}

      <div className="absolute top-2 right-2 z-10 flex flex-col gap-1">
        <button
          className="flex h-7 w-7 items-center justify-center border text-sm"
          style={{
            background: "#141414",
            borderColor: "#262626",
            color: "#e5e5e5",
          }}
          onClick={() =>
            setPosition((p) => ({ ...p, zoom: Math.min(p.zoom * 1.5, 8) }))
          }
        >
          +
        </button>
        <button
          className="flex h-7 w-7 items-center justify-center border text-sm"
          style={{
            background: "#141414",
            borderColor: "#262626",
            color: "#e5e5e5",
          }}
          onClick={() =>
            setPosition((p) => ({ ...p, zoom: Math.max(p.zoom / 1.5, 1) }))
          }
        >
          −
        </button>
      </div>

      <ComposableMap projection="geoNaturalEarth1">
        <ZoomableGroup
          zoom={position.zoom}
          center={position.coordinates}
          onMoveEnd={handleMoveEnd}
          maxZoom={8}
        >
          <Geographies geography={worldTopology}>
            {({ geographies }) =>
              geographies.map((geo) => (
                <Geography
                  key={geo.rsmKey}
                  geography={geo}
                  fill="var(--chart-3)"
                  stroke="var(--border)"
                  strokeWidth={0.9}
                  style={{
                    default: { outline: "none" },
                    hover: { outline: "none", fill: "var(--chart-4)" },
                    pressed: { outline: "none" },
                  }}
                />
              ))
            }
          </Geographies>

          {pins.map((pin) => {
            const r = (4 + (pin.total / maxTotal) * 6) / position.zoom
            return (
              <Marker key={pin.code} coordinates={pin.coords}>
                <circle
                  r={r}
                  fill="#1D9E75"
                  stroke="#0F6E56"
                  strokeWidth={1.5 / position.zoom}
                  className="cursor-pointer"
                  onMouseEnter={(e) => showTooltip(e, pin)}
                  onMouseMove={(e) => showTooltip(e, pin)}
                  onMouseLeave={() => setTooltip(null)}
                />
              </Marker>
            )
          })}
        </ZoomableGroup>
      </ComposableMap>
    </div>
  )
}

export default function Countries() {
  return (
    <DashboardPage>
      <DashboardCard className="w-full">
        <CardHeader>
          <CardDescription>See where your DNS requests lead to</CardDescription>
          <CardTitle>Countries</CardTitle>
        </CardHeader>
        <CardContent>
          <WorldMap />
        </CardContent>
      </DashboardCard>
    </DashboardPage>
  )
}
