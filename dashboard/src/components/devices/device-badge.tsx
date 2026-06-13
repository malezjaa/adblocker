import { DEVICE_CONFIG } from "@/components/devices/device-table-config.ts"
import type { Device } from "@/lib/types.ts"
import { cn } from "@/lib/utils.ts"

export function DeviceBadge({ device }: { device: Device }) {
  const config = DEVICE_CONFIG[device.device_type]
  const Icon = config.icon

  return (
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
        <p className="font-mono text-xs text-muted-foreground">{device.id}</p>
      </div>
    </div>
  )
}
