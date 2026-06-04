"use client"

import { useMemo, useState } from "react"
import {
  Apple,
  CirclePlusIcon,
  EllipsisVertical,
  FolderMinus,
  FolderPen,
  FolderPlus,
  HelpCircle,
  Monitor,
  Router,
  SearchIcon,
  Smartphone,
  Tablet,
  Terminal,
  Wifi,
  WifiOff,
} from "lucide-react"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import { Checkbox } from "@/components/ui/checkbox"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card"
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group"
import { cn } from "@/lib/utils"
import { DeviceType, post, useDevices } from "@/lib/api.ts"
import { Button } from "@/components/ui/button"
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { toast } from "sonner"
import { useQueryClient } from "@tanstack/react-query"

const DEVICE_CONFIG: Record<
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

const TABLE_ACTIONS = [
  { icon: FolderPlus, label: "Add" },
  { icon: FolderPen, label: "Edit" },
  { icon: FolderMinus, label: "Delete" },
]

function formatLastSeen(ts: number): { label: string; isRecent: boolean } {
  const diff = Date.now() - ts
  const mins = Math.floor(diff / 60_000)
  const hours = Math.floor(diff / 3_600_000)
  const days = Math.floor(diff / 86_400_000)

  if (mins < 1) return { label: "Just now", isRecent: true }
  if (mins < 60) return { label: `${mins}m ago`, isRecent: mins < 5 }
  if (hours < 24) return { label: `${hours}h ago`, isRecent: false }
  return { label: `${days}d ago`, isRecent: false }
}

const DeviceTable = () => {
  const { data: devices = [], isLoading, isError } = useDevices()
  const [open, setOpen] = useState(false)
  const [search, setSearch] = useState("")
  const [deviceName, setDeviceName] = useState("")
  const [deviceType, setDeviceType] = useState<DeviceType>(DeviceType.Windows)
  const [nameError, setNameError] = useState("")
  const queryClient = useQueryClient()

  const filtered = useMemo(() => {
    const q = search.trim().toLowerCase()
    if (!q) return devices
    return devices.filter(
      (d) =>
        d.name.toLowerCase().includes(q) ||
        DEVICE_CONFIG[d.device_type].label.toLowerCase().includes(q)
    )
  }, [devices, search])

  const submit = async () => {
    let response = await post<{ id: string } | { error: string }>(
      "api/devices",
      { name: deviceName, device_type: deviceType }
    )

    console.log(response)
    if ("error" in response) {
      setNameError(response.error)
    } else {
      toast.success("Device registered successfully")
      await queryClient.invalidateQueries({ queryKey: ["devices"] })
      setSearch("")
      setDeviceName("")
      setDeviceType(DeviceType.Windows)
      setOpen(false)
    }
  }

  return (
    <Card className="h-full w-full gap-6 pt-6 pb-0">
      <CardHeader className="items-center justify-between px-6 sm:flex">
        <div>
          <CardTitle className="leading-normal">Connected Devices</CardTitle>
          <CardDescription>
            All registered devices and their last activity
          </CardDescription>
        </div>
        <div className={"flex flex-row gap-2"}>
          <Dialog open={open} onOpenChange={setOpen}>
            <Button onClick={() => setOpen(true)}>
              <CirclePlusIcon />
              Add new device
            </Button>
            <DialogContent className="gap-0 rounded-3xl p-0 sm:max-w-lg">
              <DialogHeader className="border-b px-6 py-4">
                <DialogTitle className="font-medium text-balance">
                  Add new device
                </DialogTitle>
                <p className="mt-0.5 text-sm text-muted-foreground">
                  Register a new device to monitor its activity.
                </p>
              </DialogHeader>

              <div className="px-6 pt-4 pb-6">
                <div className="flex flex-col justify-between">
                  <div className="space-y-4">
                    <div className="space-y-2">
                      <Label htmlFor="device-name">
                        Device name <span className="text-primary">*</span>
                      </Label>
                      <Input
                        id="device-name"
                        value={deviceName}
                        onChange={(e) => setDeviceName(e.target.value)}
                        className={cn(
                          nameError &&
                            "border-destructive focus-visible:ring-destructive"
                        )}
                      />
                      {nameError && (
                        <p className="text-sm text-destructive">{nameError}</p>
                      )}
                    </div>

                    <div className="space-y-2">
                      <Label htmlFor="device-type">
                        Device Type <span className="text-primary">*</span>
                      </Label>
                      <Select
                        value={deviceType}
                        onValueChange={(v) => setDeviceType(v as DeviceType)}
                      >
                        <SelectTrigger id="device-type" className="w-full">
                          <SelectValue>
                            {(() => {
                              const cfg = DEVICE_CONFIG[deviceType]
                              const Icon = cfg.icon
                              return (
                                <span className="flex items-center gap-2">
                                  <span
                                    className={cn(
                                      "flex h-5 w-5 shrink-0 items-center justify-center rounded-full",
                                      cfg.iconBg
                                    )}
                                  >
                                    <Icon
                                      width={11}
                                      height={11}
                                      className={cfg.iconColor}
                                    />
                                  </span>
                                  {cfg.label}
                                </span>
                              )
                            })()}
                          </SelectValue>
                        </SelectTrigger>
                        <SelectContent className="w-[--radix-select-trigger-width] p-1">
                          {(
                            Object.entries(DEVICE_CONFIG) as [
                              DeviceType,
                              (typeof DEVICE_CONFIG)[DeviceType],
                            ][]
                          ).map(([type, cfg]) => {
                            const Icon = cfg.icon
                            return (
                              <SelectItem key={type} value={type}>
                                <span className="flex items-center gap-2">
                                  <span
                                    className={cn(
                                      "flex h-5 w-5 shrink-0 items-center justify-center rounded-full",
                                      cfg.iconBg
                                    )}
                                  >
                                    <Icon
                                      width={11}
                                      height={11}
                                      className={cfg.iconColor}
                                    />
                                  </span>
                                  {cfg.label}
                                </span>
                              </SelectItem>
                            )
                          })}
                        </SelectContent>
                      </Select>
                    </div>
                  </div>

                  <div className="mt-3 flex justify-end gap-2">
                    <Button variant="outline" onClick={() => setOpen(false)}>
                      Cancel
                    </Button>
                    <Button
                      className="bg-foreground text-background hover:bg-foreground/90"
                      onClick={submit}
                    >
                      Save Changes
                    </Button>
                  </div>
                </div>
              </div>
            </DialogContent>
          </Dialog>

          <InputGroup className="h-9 w-fit">
            <InputGroupInput
              placeholder="Search devices…"
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
            <InputGroupAddon>
              <SearchIcon size={18} />
            </InputGroupAddon>
          </InputGroup>
        </div>
      </CardHeader>

      <CardContent className="px-0">
        <div className="overflow-x-auto">
          <Table className="min-w-2xl">
            <TableHeader>
              <TableRow className="hover:bg-transparent!">
                <TableHead className="p-3 ps-6">#</TableHead>
                <TableHead className="p-2">Device Name</TableHead>
                <TableHead className="p-2">Type</TableHead>
                <TableHead className="p-2">Last Seen</TableHead>
                <TableHead className="flex justify-end p-3 pe-6">
                  Action
                </TableHead>
              </TableRow>
            </TableHeader>

            <TableBody className="dark:divide-darkborder divide-y divide-border">
              {isLoading && (
                <TableRow>
                  <TableCell
                    colSpan={5}
                    className="py-10 text-center text-muted-foreground"
                  >
                    Loading devices…
                  </TableCell>
                </TableRow>
              )}

              {isError && (
                <TableRow>
                  <TableCell
                    colSpan={5}
                    className="py-10 text-center text-destructive"
                  >
                    Failed to load devices.
                  </TableCell>
                </TableRow>
              )}

              {!isLoading && !isError && filtered.length === 0 && (
                <TableRow>
                  <TableCell
                    colSpan={5}
                    className="py-10 text-center text-muted-foreground"
                  >
                    No devices match your search.
                  </TableCell>
                </TableRow>
              )}

              {filtered.map((device, _) => {
                const config = DEVICE_CONFIG[device.device_type]
                const Icon = config.icon
                const { label: lastSeenLabel, isRecent } = formatLastSeen(
                  device.last_seen
                )

                return (
                  <TableRow key={device.id}>
                    <TableCell className="p-3 ps-6 whitespace-nowrap">
                      <Checkbox className="cursor-pointer data-[state=checked]:border-blue-500 data-[state=checked]:bg-blue-500 dark:data-[state=checked]:border-blue-500 dark:data-[state=checked]:bg-blue-500" />
                    </TableCell>

                    <TableCell className="whitespace-nowrap">
                      <div className="flex items-center gap-2">
                        <div
                          className={cn(
                            "flex h-9 w-9 items-center justify-center rounded-full",
                            config.iconBg
                          )}
                        >
                          <Icon
                            width={18}
                            height={18}
                            className={cn(config.iconColor)}
                          />
                        </div>
                        <div>
                          <h6 className="text-sm font-medium">{device.name}</h6>
                          <p className="font-mono text-xs text-muted-foreground">
                            {device.id}
                          </p>
                        </div>
                      </div>
                    </TableCell>

                    <TableCell className="whitespace-nowrap">
                      <span
                        className={cn(
                          "inline-flex items-center gap-1.5 rounded-full px-2.5 py-0.5 text-xs font-medium",
                          config.badgeColor
                        )}
                      >
                        <Icon width={11} height={11} />
                        {config.label}
                      </span>
                    </TableCell>

                    <TableCell className="whitespace-nowrap">
                      <div className="flex items-center gap-1.5">
                        {isRecent ? (
                          <Wifi
                            width={14}
                            height={14}
                            className="text-emerald-400"
                          />
                        ) : (
                          <WifiOff
                            width={14}
                            height={14}
                            className="text-muted-foreground"
                          />
                        )}
                        <span
                          className={cn(
                            "text-sm",
                            isRecent
                              ? "font-medium text-emerald-400"
                              : "text-muted-foreground"
                          )}
                        >
                          {lastSeenLabel}
                        </span>
                      </div>
                    </TableCell>

                    <TableCell className="p-3 pe-6 whitespace-nowrap">
                      <div className="flex items-center justify-end">
                        <DropdownMenu>
                          <DropdownMenuTrigger>
                            <span className="flex cursor-pointer items-center justify-center rounded-full p-2 hover:bg-muted">
                              <EllipsisVertical width={16} height={16} />
                            </span>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            {TABLE_ACTIONS.map((action, idx) => (
                              <DropdownMenuItem
                                key={idx}
                                className="group flex cursor-pointer gap-3 hover:bg-accent!"
                              >
                                <action.icon />
                                <span>{action.label}</span>
                              </DropdownMenuItem>
                            ))}
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>
                    </TableCell>
                  </TableRow>
                )
              })}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}

export default DeviceTable
