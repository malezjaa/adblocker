import { cn } from "@/lib/utils.ts"
import { SidebarInset, SidebarProvider } from "@/components/ui/sidebar.tsx"
import { AppHeader } from "@/components/app/app-header.tsx"
import { AppSidebar } from "@/components/app/app-sidebar.tsx"

export function AppShell({ children }: { children: React.ReactNode }) {
  return (
    <SidebarProvider className={cn("[--app-wrapper-max-width:80rem]")}>
      <AppSidebar />
      <SidebarInset>
        <AppHeader />
        <div
          className={cn(
            "flex flex-1 flex-col p-4 md:p-6",
            "mx-auto w-full max-w-(--app-wrapper-max-width)"
          )}
        >
          {children}
        </div>
      </SidebarInset>
    </SidebarProvider>
  )
}
