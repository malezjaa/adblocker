import { Dashboard } from "@/app/dashboard/dashboard.tsx"
import { createBrowserRouter, RouterProvider } from "react-router"
import { NotFoundPage } from "@/components/not-found.tsx"
import { AuthPage } from "@/components/auth.tsx"
import { QueryLogs } from "@/app/dashboard/query-logs.tsx"
import Countries from "@/app/dashboard/countries.tsx"

let router = createBrowserRouter([
  {
    path: "*",
    Component: NotFoundPage,
  },
  {
    path: "/dashboard",
    Component: Dashboard,
  },
  {
    path: "/query-logs",
    Component: QueryLogs,
  },
  {
    path: "/login",
    Component: AuthPage,
  },
  {
    path: "/countries",
    Component: Countries,
  },
])

export function App() {
  return <RouterProvider router={router} />
}

export default App
