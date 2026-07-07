import {
  ComposableMap,
  Geographies,
  Geography,
  Marker,
  ZoomableGroup,
} from "react-simple-maps"
import React, { useRef, useState } from "react"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import { useStats } from "@/lib/api.ts"

type TooltipState = {
  name: string
  info: string
  x: number
  y: number
} | null

const GEO_URL = "https://cdn.jsdelivr.net/npm/world-atlas@2/countries-110m.json"

const COUNTRY_COORDS: Record<string, [number, number]> = {
  US: [-95.71, 37.09],
  GB: [-3.44, 55.38],
  DE: [10.45, 51.17],
  FR: [2.21, 46.23],
  JP: [138.25, 36.2],
  CN: [104.19, 35.86],
  BR: [-51.93, -14.24],
  IN: [78.96, 20.59],
  RU: [105.32, 61.52],
  AU: [133.77, -25.27],
  CA: [-96.8, 56.13],
  MX: [-102.55, 23.63],
  ZA: [25.08, -29.0],
  NG: [8.68, 9.08],
  EG: [30.8, 26.82],
  KR: [127.77, 35.91],
  ID: [113.92, -0.79],
  SA: [45.08, 23.89],
  AR: [-63.62, -38.42],
  TR: [35.24, 38.96],
  IT: [12.57, 41.87],
  ES: [-3.75, 40.46],
  PL: [19.15, 51.92],
  NL: [5.29, 52.13],
  SE: [18.64, 60.13],
  NO: [8.47, 60.47],
  CH: [8.23, 46.82],
  BE: [4.47, 50.5],
  AT: [14.55, 47.52],
  PT: [-8.22, 39.4],
  CZ: [15.47, 49.82],
  UA: [31.17, 48.38],
  RO: [24.97, 45.94],
  HU: [19.5, 47.16],
  GR: [21.82, 39.07],
  FI: [25.75, 61.92],
  DK: [10.0, 56.26],
  TH: [100.99, 15.87],
  VN: [108.28, 14.06],
  PH: [122.88, 12.88],
  MY: [109.7, 4.21],
  SG: [103.82, 1.36],
  NZ: [174.89, -40.9],
  CL: [-71.54, -35.68],
  CO: [-74.3, 4.57],
  PE: [-75.02, -9.19],
  PK: [69.35, 30.38],
  BD: [90.36, 23.68],
  IR: [53.69, 32.43],
  IQ: [43.68, 33.22],
  IL: [34.85, 31.05],
  AE: [53.85, 23.42],
  KE: [37.91, -0.02],
  TZ: [34.89, -6.37],
  GH: [-1.02, 7.95],
  ET: [40.49, 9.15],
  MA: [-7.09, 31.79],
  TW: [120.96, 23.7],
  HK: [114.11, 22.4],
}

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
    .filter((c) => COUNTRY_COORDS[c.country_code])
    .map((c) => ({
      name: getCountryName(c.country_code),
      coords: COUNTRY_COORDS[c.country_code],
      total: c.total,
      blocked: c.blocked,
    }))

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
          <Geographies geography={GEO_URL}>
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
              <Marker key={pin.name} coordinates={pin.coords}>
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
