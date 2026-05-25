import { Dashboard } from "@/app/dashboard/dashboard.tsx"
import { useStatsWs } from "@/lib/api.ts"

export function App() {
  useStatsWs()

  return <Dashboard />
}

export default App
