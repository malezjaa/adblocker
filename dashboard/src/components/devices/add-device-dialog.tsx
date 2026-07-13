"use client"

import { useState } from "react"
import { CirclePlusIcon } from "lucide-react"
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
import { cn } from "@/lib/utils"
import { post } from "@/lib/api"
import { toast } from "sonner"
import { useQueryClient } from "@tanstack/react-query"
import { DEVICE_CONFIG } from "./device-table-config"
import { type DeviceType, DeviceTypes } from "@/lib/types.ts"

export function AddDeviceDialog() {
  const [open, setOpen] = useState(false)
  const [deviceName, setDeviceName] = useState("")
  const [deviceType, setDeviceType] = useState<DeviceType>(DeviceTypes.Windows)
  const [nameError, setNameError] = useState("")
  const queryClient = useQueryClient()

  const handleSubmit = async () => {
    const response = await post<
      { id: string; restored: boolean } | { error: string }
    >("api/devices", { name: deviceName, device_type: deviceType })

    if ("error" in response) {
      setNameError(response.error)
    } else {
      toast.success(
        response.restored
          ? "Device restored with its previous ID"
          : "Device registered successfully"
      )
      await queryClient.invalidateQueries({ queryKey: ["devices"] })
      setDeviceName("")
      setDeviceType(DeviceTypes.Windows)
      setNameError("")
      setOpen(false)
    }
  }

  const handleOpenChange = (next: boolean) => {
    setOpen(next)
    if (!next) {
      setDeviceName("")
      setDeviceType(DeviceTypes.Windows)
      setNameError("")
    }
  }

  return (
    <Dialog open={open} onOpenChange={handleOpenChange}>
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
                  onChange={(e) => {
                    setDeviceName(e.target.value)
                    if (nameError) setNameError("")
                  }}
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
              <Button variant="outline" onClick={() => handleOpenChange(false)}>
                Cancel
              </Button>
              <Button
                className="bg-foreground text-background hover:bg-foreground/90"
                onClick={handleSubmit}
              >
                Save Changes
              </Button>
            </div>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  )
}
