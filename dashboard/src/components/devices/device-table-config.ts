import {Apple, HelpCircle, Monitor, Router, Smartphone, Tablet, Terminal,} from "lucide-react"
import {DeviceType} from "@/lib/api"

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
  [DeviceType.Windows]: {
    label: "Windows",
    icon: Monitor,
    iconColor: "text-sky-400",
    iconBg: "bg-sky-400/15",
    badgeColor: "bg-sky-400/10 text-sky-400 ring-1 ring-sky-400/30",
  },
  [DeviceType.Linux]: {
    label: "Linux",
    icon: Terminal,
    iconColor: "text-amber-400",
    iconBg: "bg-amber-400/15",
    badgeColor: "bg-amber-400/10 text-amber-400 ring-1 ring-amber-400/30",
  },
  [DeviceType.MacOs]: {
    label: "macOS",
    icon: Apple,
    iconColor: "text-violet-400",
    iconBg: "bg-violet-400/15",
    badgeColor: "bg-violet-400/10 text-violet-400 ring-1 ring-violet-400/30",
  },
  [DeviceType.Android]: {
    label: "Android",
    icon: Smartphone,
    iconColor: "text-emerald-400",
    iconBg: "bg-emerald-400/15",
    badgeColor: "bg-emerald-400/10 text-emerald-400 ring-1 ring-emerald-400/30",
  },
  [DeviceType.iOS]: {
    label: "iOS",
    icon: Tablet,
    iconColor: "text-rose-400",
    iconBg: "bg-rose-400/15",
    badgeColor: "bg-rose-400/10 text-rose-400 ring-1 ring-rose-400/30",
  },
  [DeviceType.Router]: {
    label: "Router",
    icon: Router,
    iconColor: "text-orange-400",
    iconBg: "bg-orange-400/15",
    badgeColor: "bg-orange-400/10 text-orange-400 ring-1 ring-orange-400/30",
  },
  [DeviceType.Other]: {
    label: "Other",
    icon: HelpCircle,
    iconColor: "text-slate-400",
    iconBg: "bg-slate-400/15",
    badgeColor: "bg-slate-400/10 text-slate-400 ring-1 ring-slate-400/30",
  },
}

export function formatLastSeen(ts: number): {
  label: string
  isRecent: boolean
} {
  const diff = Date.now() - ts * 1000;
  const mins = Math.floor(diff / 60_000)
  const hours = Math.floor(diff / 3_600_000)
  const days = Math.floor(diff / 86_400_000)

  if (mins < 1) return { label: "Just now", isRecent: true }
  if (mins < 60) return { label: `${mins}m ago`, isRecent: mins < 5 }
  if (hours < 24) return { label: `${hours}h ago`, isRecent: false }
  return { label: `${days}d ago`, isRecent: false }
}
