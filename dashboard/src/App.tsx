import { Dashboard } from "@/app/dashboard/dashboard.tsx"
import { createBrowserRouter, RouterProvider } from "react-router"
import { NotFoundPage } from "@/components/not-found.tsx"
import { AuthPage } from "@/components/auth.tsx"
import { QueryLogs } from "@/app/dashboard/query-logs.tsx"
import Countries from "@/app/dashboard/countries.tsx"
import { Lists } from "@/app/dashboard/lists.tsx"
import Settings from "@/app/dashboard/settings/settings.tsx"
import { HTML5Backend } from "react-dnd-html5-backend"
import { DndProvider } from "react-dnd"
import Rules from "@/app/dashboard/rules.tsx"

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
  {
    path: "/lists",
    Component: Lists,
  },
  {
    path: "/settings",
    Component: Settings,
  },
  {
    path: "/rules",
    Component: Rules,
  },
])

export function App() {
  return (
    <>
      <DndProvider backend={HTML5Backend}>
        <RouterProvider router={router} />
      </DndProvider>
    </>
  )
}

export default App
