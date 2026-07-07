import { RequestsChart } from "@/app/dashboard/components/requests-chart.tsx"
import { SectionCards } from "@/app/dashboard/components/section-cards.tsx"
import { TopEntitiesCards } from "@/app/dashboard/components/top-entities-cards.tsx"
import Devices from "@/components/devices/devices.tsx"
import { useStatsWs } from "@/lib/api.ts"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"

export function Dashboard() {
  useStatsWs()

  return (
    <DashboardPage className="border-collapse">
      <SectionCards />
      <div className="px-4 lg:px-6">
        <RequestsChart />
      </div>

      <TopEntitiesCards />

      <div className="px-4 lg:px-6">
        <Devices />
      </div>
    </DashboardPage>
  )
}

export default Dashboard
