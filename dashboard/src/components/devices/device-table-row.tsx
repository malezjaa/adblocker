"use client"

import {
  EllipsisVertical,
  FolderMinus,
  FolderPen,
  Wifi,
  WifiOff,
} from "lucide-react"
import { TableCell, TableRow } from "@/components/ui/table"
import { Checkbox } from "@/components/ui/checkbox"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import { cn } from "@/lib/utils"
import { type Device } from "@/lib/api"
import { DEVICE_CONFIG, formatLastSeen } from "./device-table-config"

interface DeviceTableRowProps {
  device: Device
  onDelete: (id: string) => void
}

export function DeviceTableRow({ device, onDelete }: DeviceTableRowProps) {
  const config = DEVICE_CONFIG[device.device_type]
  const Icon = config.icon
  const { label: lastSeenLabel, isRecent } = formatLastSeen(device.last_seen)

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
            <Icon width={18} height={18} className={cn(config.iconColor)} />
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
            <Wifi width={14} height={14} className="text-emerald-400" />
          ) : (
            <WifiOff width={14} height={14} className="text-muted-foreground" />
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
              <DropdownMenuItem className="flex cursor-pointer gap-3 hover:bg-accent!">
                <FolderPen />
                <span>Edit</span>
              </DropdownMenuItem>
              <DropdownMenuItem
                className="flex cursor-pointer gap-3 text-destructive focus:bg-destructive/10 focus:text-destructive"
                onClick={() => onDelete(device.id)}
              >
                <FolderMinus />
                <span>Delete</span>
              </DropdownMenuItem>
            </DropdownMenuContent>
          </DropdownMenu>
        </div>
      </TableCell>
    </TableRow>
  )
}
