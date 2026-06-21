import { cn } from "@/lib/utils.ts"
import { Button } from "@/components/ui/button.tsx"
import { Separator } from "@/components/ui/separator.tsx"
import { DecorIcon } from "../ui/decor-icon.tsx"
import { CustomSidebarTrigger } from "@/components/custom-sidebar-trigger"
import { NavUser } from "../nav/nav-user.tsx"
import { BellIcon, SendIcon } from "lucide-react"

export function AppHeader() {
  return (
    <header
      className={cn(
        "sticky top-0 z-50 flex h-14 shrink-0 items-center justify-between gap-2 border-b px-4 md:px-6",
        "bg-background/95 backdrop-blur-sm supports-backdrop-filter:bg-background/50"
      )}
    >
      <DecorIcon className="hidden md:block" position="bottom-left" />
      <div className="flex items-center gap-3">
        <CustomSidebarTrigger />
        <Separator
          className="mr-2 h-4 data-[orientation=vertical]:self-center"
          orientation="vertical"
        />
      </div>
      <div className="flex items-center gap-3">
        <Button size="icon-sm" variant="outline">
          <SendIcon />
        </Button>
        <Button aria-label="Notifications" size="icon-sm" variant="outline">
          <BellIcon />
        </Button>
        <Separator
          className="h-4 data-[orientation=vertical]:self-center"
          orientation="vertical"
        />
        <NavUser />
      </div>
    </header>
  )
}
