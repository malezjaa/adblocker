"use client"

import { useMemo, useState } from "react"
import { SearchIcon } from "lucide-react"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
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
import { del, useDevices } from "@/lib/api"
import { toast } from "sonner"
import { useQueryClient } from "@tanstack/react-query"
import { AddDeviceDialog } from "./add-device-dialog"
import { DeviceTableRow } from "./device-table-row"
import { DEVICE_CONFIG } from "./device-table-config"

const DeviceTable = () => {
  const { data: devices = [], isLoading, isError } = useDevices()
  const [search, setSearch] = useState("")
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

  const handleDelete = async (id: string) => {
    const response = await del<{ error?: string }>(`api/devices/${id}`)
    if (response?.error) {
      toast.error(response.error)
    } else {
      toast.success("Device deleted successfully")
      await queryClient.invalidateQueries({ queryKey: ["devices"] })
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
        <div className="flex flex-row gap-2">
          <AddDeviceDialog />
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
              {filtered.map((device) => (
                <DeviceTableRow
                  key={device.id}
                  device={device}
                  onDelete={handleDelete}
                />
              ))}
            </TableBody>
          </Table>
        </div>
      </CardContent>
    </Card>
  )
}

export default DeviceTable
