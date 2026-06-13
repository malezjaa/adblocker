import type { ReactNode } from "react"
import {
  BookOpenIcon,
  BriefcaseIcon,
  Flag,
  HelpCircleIcon,
  LayoutGridIcon,
  List,
  Pen,
  Scroll,
} from "lucide-react"

export type SidebarNavItem = {
  title: string
  path?: string
  icon?: ReactNode
  subItems?: SidebarNavItem[]
}

export type SidebarNavGroup = {
  label?: string
  items: SidebarNavItem[]
}

export const navGroups: SidebarNavGroup[] = [
  {
    label: "Dashboard",
    items: [
      {
        title: "Dashboard",
        path: "/dashboard",
        icon: <LayoutGridIcon />,
      },
      {
        title: "Countries",
        path: "/countries",
        icon: <Flag />,
      },
      {
        title: "Companies",
        path: "/companies",
        icon: <BriefcaseIcon />,
      },
      {
        title: "Query Logs",
        path: "/query-logs",
        icon: <Scroll />,
      },
    ],
  },
  {
    label: "Config",
    items: [
      {
        title: "Lists",
        path: "/lists",
        icon: <List />,
      },
      {
        title: "Rewrites",
        path: "/rewrites",
        icon: <Pen />,
      },
    ],
  },
]

export const footerNavLinks: SidebarNavItem[] = [
  {
    title: "Help Center",
    path: "#/help",
    icon: <HelpCircleIcon />,
  },
  {
    title: "Documentation",
    path: "#/documentation",
    icon: <BookOpenIcon />,
  },
]

export const navLinks: SidebarNavItem[] = [
  ...navGroups.flatMap((group) =>
    group.items.flatMap((item) =>
      item.subItems?.length ? [item, ...item.subItems] : [item]
    )
  ),
  ...footerNavLinks,
]
