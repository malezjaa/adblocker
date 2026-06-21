import { useRef, useState } from "react"
import { useDrag, useDrop } from "react-dnd"
import { Button } from "@/components/ui/button.tsx"
import { Label } from "@/components/ui/label.tsx"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select.tsx"
import { GripVertical, Plus, Server, Trash } from "lucide-react"
import type { UpstreamServer } from "@/app/dashboard/settings/user-settings.ts"
import { cn } from "@/lib/utils.ts"

export const AVAILABLE_UPSTREAMS: UpstreamServer[] = [
  { name: "cloudflare-dns.com", addr: "1.1.1.1" },
  { name: "cloudflare-dns.com", addr: "1.0.0.1" },
  { name: "dns.google", addr: "8.8.8.8" },
  { name: "dns.google", addr: "8.8.4.4" },
  { name: "dns.quad9.net", addr: "9.9.9.9" },
  { name: "dns.quad9.net", addr: "149.112.112.112" },
  { name: "doh.opendns.com", addr: "208.67.222.222" },
  { name: "doh.opendns.com", addr: "208.67.220.220" },
]

function upstreamKey(server: UpstreamServer) {
  return server.addr
}

const ITEM_TYPE = "upstream-server"

interface DragItem {
  index: number
}

interface UpstreamRowProps {
  server: UpstreamServer
  index: number
  moveItem: (from: number, to: number) => void
  remove: (index: number) => void
  disabled?: boolean
}

function UpstreamRow({
  server,
  index,
  moveItem,
  remove,
  disabled,
}: UpstreamRowProps) {
  const ref = useRef<HTMLLIElement>(null)

  const [{ isDragging }, drag] = useDrag({
    type: ITEM_TYPE,
    item: (): DragItem => ({ index }),
    canDrag: !disabled,
    collect: (monitor) => ({ isDragging: monitor.isDragging() }),
  })

  const [{ isOver }, drop] = useDrop<DragItem, void, { isOver: boolean }>({
    accept: ITEM_TYPE,

    collect: (monitor) => ({
      isOver: monitor.isOver(),
    }),

    hover(item, monitor) {
      if (!ref.current) return

      const dragIndex = item.index
      const hoverIndex = index

      if (dragIndex === hoverIndex) return

      const hoverBoundingRect = ref.current.getBoundingClientRect()

      const hoverMiddleY =
        (hoverBoundingRect.bottom - hoverBoundingRect.top) / 2

      const clientOffset = monitor.getClientOffset()

      if (!clientOffset) return

      const hoverClientY = clientOffset.y - hoverBoundingRect.top

      if (dragIndex < hoverIndex && hoverClientY < hoverMiddleY) {
        return
      }

      if (dragIndex > hoverIndex && hoverClientY > hoverMiddleY) {
        return
      }

      moveItem(dragIndex, hoverIndex)

      item.index = hoverIndex
    },
  })

  drag(drop(ref))

  return (
    <li
      ref={ref}
      className={cn(
        "group flex items-center gap-3 rounded-lg border bg-card px-3 py-2.5 transition-colors hover:bg-muted/40",
        isDragging && "opacity-0",
        isOver && !isDragging && "border-primary"
      )}
    >
      <span
        className={cn(
          "flex size-6 shrink-0 cursor-grab items-center justify-center rounded-full text-muted-foreground/60 hover:text-muted-foreground active:cursor-grabbing",
          disabled && "cursor-not-allowed opacity-50"
        )}
      >
        <GripVertical className="size-4" />
      </span>

      <div className="flex min-w-0 flex-1 flex-row items-center gap-2">
        <span className="truncate text-sm leading-tight font-medium">
          {server.name}
        </span>
        <span className="truncate text-xs text-muted-foreground">
          {server.addr}
        </span>
      </div>

      <div className="flex items-center gap-0.5 opacity-60 transition-opacity group-hover:opacity-100">
        <Button
          type="button"
          variant="ghost"
          size="icon"
          className="size-7 text-muted-foreground hover:text-destructive"
          disabled={disabled}
          onClick={() => remove(index)}
          aria-label="Remove"
        >
          <Trash className="size-3.5" />
        </Button>
      </div>
    </li>
  )
}

interface UpstreamsFieldProps {
  label: string
  description?: string
  value: UpstreamServer[]
  onChange: (value: UpstreamServer[]) => void
  disabled?: boolean
}

export function UpstreamsField({
  label,
  description,
  value,
  onChange,
  disabled,
}: UpstreamsFieldProps) {
  const [pendingValue, setPendingValue] = useState("")

  const selectedKeys = new Set(value.map(upstreamKey))
  const availableToAdd = AVAILABLE_UPSTREAMS.filter(
    (server) => !selectedKeys.has(upstreamKey(server))
  )

  function moveItem(from: number, to: number) {
    const next = [...value]
    const [moved] = next.splice(from, 1)
    next.splice(to, 0, moved)
    onChange(next)
  }

  function remove(index: number) {
    onChange(value.filter((_, i) => i !== index))
  }

  function add(ip: string) {
    const server = AVAILABLE_UPSTREAMS.find((s) => s.addr === ip)
    if (!server) return
    onChange([...value, server])
    setPendingValue("")
  }

  return (
    <div className="flex flex-col gap-4 py-5 first:pt-0 last:pb-0">
      <div className="space-y-0.5">
        <Label className="text-sm font-medium">{label}</Label>
        {description ? (
          <p className="text-sm text-muted-foreground">{description}</p>
        ) : null}
      </div>

      <div className="flex flex-col gap-3">
        {value.length === 0 ? (
          <div className="flex flex-col items-center justify-center gap-2 rounded-lg border border-dashed py-8 text-center">
            <Server className="size-5 text-muted-foreground/60" />
            <p className="text-sm text-muted-foreground">
              No upstream servers configured yet.
            </p>
          </div>
        ) : (
          <ul className="flex flex-col gap-2">
            {value.map((server, index) => (
              <UpstreamRow
                key={upstreamKey(server)}
                server={server}
                index={index}
                moveItem={moveItem}
                remove={remove}
                disabled={disabled}
              />
            ))}
          </ul>
        )}

        {availableToAdd.length > 0 && (
          <div className="flex w-full items-center gap-2">
            <Select
              value={pendingValue}
              onValueChange={(val) => setPendingValue(val ?? "")}
              disabled={disabled}
            >
              <SelectTrigger className="w-full flex-1">
                <SelectValue placeholder="Add upstream server" />
              </SelectTrigger>
              <SelectContent>
                {availableToAdd.map((server) => (
                  <SelectItem key={server.addr} value={server.addr}>
                    {server.name} ({server.addr})
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Button
              type="button"
              variant="outline"
              size="icon"
              className="shrink-0"
              disabled={disabled || !pendingValue}
              onClick={() => add(pendingValue)}
              aria-label="Add server"
            >
              <Plus className="size-4" />
            </Button>
          </div>
        )}
      </div>
    </div>
  )
}
