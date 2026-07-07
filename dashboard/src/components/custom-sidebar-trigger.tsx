import { Kbd, KbdGroup } from "@/components/ui/kbd"
import { SidebarTrigger } from "@/components/ui/sidebar"
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip"

export function CustomSidebarTrigger() {
  return (
    <TooltipProvider delay={1000}>
      <Tooltip>
        <TooltipTrigger render={<SidebarTrigger />} />
        <TooltipContent className="px-2 py-1" side="right">
          Toggle Sidebar{" "}
          <KbdGroup>
            <Kbd>⌘</Kbd>
            <Kbd>b</Kbd>
          </KbdGroup>
        </TooltipContent>
      </Tooltip>
    </TooltipProvider>
  )
}
