import {
  Apple,
  HelpCircle,
  Monitor,
  Router,
  Smartphone,
  Tablet,
  Terminal,
} from "lucide-react"
import { type DeviceType, DeviceTypes } from "@/lib/types.ts"

export const DEVICE_CONFIG: Record<
  DeviceType,
  {
    label: string
    icon: React.ElementType
    iconColor: string
    iconBg: string
    badgeColor: string
  }
> = {
  [DeviceTypes.Windows]: {
    label: "Windows",
    icon: Monitor,
    iconColor: "text-blue-400",
    iconBg: "bg-blue-400/10",
    badgeColor: "bg-blue-400/10 text-blue-400 ring-1 ring-blue-400/20",
  },

  [DeviceTypes.Linux]: {
    label: "Linux",
    icon: Terminal,
    iconColor: "text-zinc-400",
    iconBg: "bg-zinc-400/10",
    badgeColor: "bg-zinc-400/10 text-zinc-400 ring-1 ring-zinc-400/20",
  },

  [DeviceTypes.MacOs]: {
    label: "macOS",
    icon: Apple,
    iconColor: "text-slate-300",
    iconBg: "bg-slate-300/10",
    badgeColor: "bg-slate-300/10 text-slate-300 ring-1 ring-slate-300/20",
  },

  [DeviceTypes.Android]: {
    label: "Android",
    icon: Smartphone,
    iconColor: "text-emerald-300",
    iconBg: "bg-emerald-300/10",
    badgeColor: "bg-emerald-300/10 text-emerald-300 ring-1 ring-emerald-300/20",
  },

  [DeviceTypes.iOS]: {
    label: "iOS",
    icon: Tablet,
    iconColor: "text-indigo-300",
    iconBg: "bg-indigo-300/10",
    badgeColor: "bg-indigo-300/10 text-indigo-300 ring-1 ring-indigo-300/20",
  },

  [DeviceTypes.Router]: {
    label: "Router",
    icon: Router,
    iconColor: "text-amber-300",
    iconBg: "bg-amber-300/10",
    badgeColor: "bg-amber-300/10 text-amber-300 ring-1 ring-amber-300/20",
  },

  [DeviceTypes.Other]: {
    label: "Other",
    icon: HelpCircle,
    iconColor: "text-muted-foreground",
    iconBg: "bg-muted",
    badgeColor: "bg-muted text-muted-foreground ring-1 ring-border",
  },
}

export function formatLastSeen(ts: number): {
  label: string
  isRecent: boolean
} {
  const diff = Date.now() - ts * 1000
  const mins = Math.floor(diff / 60_000)
  const hours = Math.floor(diff / 3_600_000)
  const days = Math.floor(diff / 86_400_000)

  if (mins < 1) return { label: "Just now", isRecent: true }
  if (mins < 60) return { label: `${mins}m ago`, isRecent: mins < 5 }
  if (hours < 24) return { label: `${hours}h ago`, isRecent: false }
  return { label: `${days}d ago`, isRecent: false }
}
