import { RequestsChart } from "@/components/requests-chart.tsx"
import { SectionCards } from "@/components/section-cards"
import { TopEntitiesCards } from "@/components/top-entities-cards"
import Devices from "@/components/devices/devices.tsx"
import { AppShell } from "@/components/app/app-shell.tsx"
import { useStatsWs } from "@/lib/api.ts"
import { ProtectedRoute } from "@/components/protected-route.tsx"

export function Dashboard() {
  useStatsWs()

  return (
    <ProtectedRoute>
      <AppShell>
        <div className="flex flex-1 flex-col">
          <div className="@container/main flex flex-1 flex-col gap-2">
            <div className="flex border-collapse flex-col py-4 md:py-6">
              <SectionCards />
              <div className="px-4 lg:px-6">
                <RequestsChart />
              </div>

              <TopEntitiesCards />

              <div className="px-4 lg:px-6">
                <Devices />
              </div>
            </div>
          </div>
        </div>
      </AppShell>
    </ProtectedRoute>
  )
}

export default Dashboard
