import { Dashboard } from "@/app/dashboard/dashboard.tsx"
import { createBrowserRouter, RouterProvider } from "react-router"
import { NotFoundPage } from "@/components/not-found.tsx"
import { AuthPage } from "@/components/auth.tsx"

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
    path: "/login",
    Component: AuthPage,
  },
])

export function App() {
  return <RouterProvider router={router} />
}

export default App
