import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible"
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar"
import type { SidebarNavGroup } from "@/components/app/app-shared.tsx"
import { ChevronRightIcon } from "lucide-react"
import { Link } from "react-router"

type NavGroupProps = SidebarNavGroup & {
  currentPath: string
}

export function NavGroup({ label, items, currentPath }: NavGroupProps) {
  const isPathActive = (path?: string) =>
    !!path && (currentPath === path || currentPath.startsWith(path + "/"))

  return (
    <SidebarGroup>
      {label && <SidebarGroupLabel>{label}</SidebarGroupLabel>}

      <SidebarMenu>
        {items.map((item) => {
          const itemActive = isPathActive(item.path)

          const subItemActive = item.subItems?.some((subItem) =>
            isPathActive(subItem.path)
          )

          const isActive = itemActive || subItemActive

          return (
            <Collapsible
              key={item.title}
              className="group/collapsible"
              defaultOpen={isActive}
              render={<SidebarMenuItem />}
            >
              {item.subItems?.length ? (
                <>
                  <CollapsibleTrigger
                    render={<SidebarMenuButton isActive={isActive} />}
                  >
                    {item.icon}
                    <span>{item.title}</span>

                    <ChevronRightIcon className="ml-auto transition-transform duration-200 group-data-[state=open]/collapsible:rotate-90" />
                  </CollapsibleTrigger>

                  <CollapsibleContent>
                    <SidebarMenuSub>
                      {item.subItems.map((subItem) => (
                        <SidebarMenuSubItem key={subItem.title}>
                          <SidebarMenuSubButton
                            isActive={isPathActive(subItem.path)}
                            render={<Link to={subItem.path || "/"} />}
                          >
                            {subItem.icon}
                            <span>{subItem.title}</span>
                          </SidebarMenuSubButton>
                        </SidebarMenuSubItem>
                      ))}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </>
              ) : (
                <SidebarMenuButton
                  isActive={isActive}
                  render={<Link to={item.path || ""} />}
                >
                  {item.icon}
                  <span>{item.title}</span>
                </SidebarMenuButton>
              )}
            </Collapsible>
          )
        })}
      </SidebarMenu>
    </SidebarGroup>
  )
}
