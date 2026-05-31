import * as React from "react"
import { format } from "date-fns"
import {
  type Column,
  type ColumnDef,
  flexRender,
  getCoreRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  type SortingState,
  useReactTable,
} from "@tanstack/react-table"

import { Button } from "@/components/ui/button"
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table"
import {
  ChevronLeftIcon,
  ChevronRightIcon,
  ChevronsLeftIcon,
  ChevronsRightIcon,
  ChevronUpIcon,
  ChevronDownIcon,
  ChevronsUpDownIcon,
} from "lucide-react"
import { Label } from "@/components/ui/label"
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select"
import { type TopBlocked, useTopBlocked } from "@/lib/api.ts"
import { formatNum } from "@/lib/utils.ts"

function SortIcon({ column }: { column: Column<TopBlocked> }) {
  const sorted = column.getIsSorted()
  if (sorted === "asc")
    return <ChevronUpIcon className="ml-1 inline h-3 w-3 shrink-0" />
  if (sorted === "desc")
    return <ChevronDownIcon className="ml-1 inline h-3 w-3 shrink-0" />
  return (
    <ChevronsUpDownIcon className="ml-1 inline h-3 w-3 shrink-0 opacity-40" />
  )
}

const columns: ColumnDef<TopBlocked>[] = [
  {
    accessorKey: "domain",
    size: 300,
    header: "Blocked domain",
    cell: ({ row }) => (
      <span className="font-medium">{row.original.domain}</span>
    ),
  },
  {
    accessorKey: "hits_blocked",
    size: 120,
    header: ({ column }) => (
      <Button
        variant="ghost"
        className="-mx-4 w-[calc(100%+1.8rem)] justify-end"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Blocked <SortIcon column={column} />
      </Button>
    ),
    sortingFn: "basic",
    cell: ({ row }) => (
      <div className="text-right">
        {row.original.hits_blocked.toLocaleString()}
      </div>
    ),
  },
  {
    accessorKey: "hits_total",
    size: 120,
    header: ({ column }) => (
      <Button
        variant="ghost"
        className="-mx-4 w-[calc(100%+1.8rem)] justify-end"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Total <SortIcon column={column} />
      </Button>
    ),
    sortingFn: "basic",
    cell: ({ row }) => (
      <div className="text-right">
        {row.original.hits_total.toLocaleString()}
      </div>
    ),
  },
  {
    accessorKey: "avg_response_time",
    size: 140,
    header: ({ column }) => (
      <Button
        variant="ghost"
        className="-mx-4 w-[calc(100%+1.8rem)] justify-end"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Average Response time <SortIcon column={column} />
      </Button>
    ),
    sortingFn: "basic",
    cell: ({ row }) => (
      <div className="text-right">
        {formatNum(row.original.avg_response_time).toLocaleString()}ms
      </div>
    ),
  },
  {
    accessorKey: "last_seen",
    size: 140,
    header: ({ column }) => (
      <Button
        variant="ghost"
        className="-mx-4 w-[calc(100%+1.8rem)] justify-end"
        onClick={() => column.toggleSorting(column.getIsSorted() === "asc")}
      >
        Last seen <SortIcon column={column} />
      </Button>
    ),
    sortingFn: "basic",
    cell: ({ row }) => (
      <div className="text-right">
        {format(new Date(row.original.last_seen * 1000), "HH:mm dd.MM.yyyy")}
      </div>
    ),
  },
]

export function DataTable() {
  const { data = [], isLoading, isError } = useTopBlocked()
  const [sorting, setSorting] = React.useState<SortingState>([])
  const [pagination, setPagination] = React.useState({
    pageIndex: 0,
    pageSize: 10,
  })

  const table = useReactTable({
    data,
    columns,
    state: { sorting, pagination },
    onSortingChange: setSorting,
    onPaginationChange: setPagination,
    getCoreRowModel: getCoreRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
  })

  if (isLoading) {
    return (
      <div className="flex h-24 items-center justify-center text-sm text-muted-foreground">
        Loading...
      </div>
    )
  }

  if (isError) {
    return (
      <div className="flex h-24 items-center justify-center text-sm text-destructive">
        Failed to load data.
      </div>
    )
  }

  return (
    <div className="relative flex flex-col gap-4 overflow-auto px-4 lg:px-6">
      <div className="flex flex-col gap-4">
        <div className="overflow-hidden rounded-lg border">
          <Table className="w-full table-fixed">
            <colgroup>
              <col className="w-75" />
              <col className="w-30" />
              <col className="w-30" />
              <col className="w-40" />
              <col className="w-40" />
            </colgroup>
            <TableHeader className="sticky top-0 z-10 bg-muted">
              {table.getHeaderGroups().map((headerGroup) => (
                <TableRow key={headerGroup.id}>
                  {headerGroup.headers.map((header) => (
                    <TableHead key={header.id} colSpan={header.colSpan}>
                      {header.isPlaceholder
                        ? null
                        : flexRender(
                            header.column.columnDef.header,
                            header.getContext()
                          )}
                    </TableHead>
                  ))}
                </TableRow>
              ))}
            </TableHeader>
            <TableBody>
              {table.getRowModel().rows.length ? (
                table.getRowModel().rows.map((row) => (
                  <TableRow key={row.id}>
                    {row.getVisibleCells().map((cell) => (
                      <TableCell key={cell.id}>
                        {flexRender(
                          cell.column.columnDef.cell,
                          cell.getContext()
                        )}
                      </TableCell>
                    ))}
                  </TableRow>
                ))
              ) : (
                <TableRow>
                  <TableCell
                    colSpan={columns.length}
                    className="h-24 text-center"
                  >
                    No results.
                  </TableCell>
                </TableRow>
              )}
            </TableBody>
          </Table>
        </div>

        <div className="flex w-full items-center gap-8 px-4 lg:w-fit lg:px-0">
          <div className="hidden items-center gap-2 lg:flex">
            <Label htmlFor="rows-per-page" className="text-sm font-medium">
              Rows per page
            </Label>
            <Select
              value={`${table.getState().pagination.pageSize}`}
              onValueChange={(value) => table.setPageSize(Number(value))}
            >
              <SelectTrigger size="sm" className="w-20" id="rows-per-page">
                <SelectValue
                  placeholder={table.getState().pagination.pageSize}
                />
              </SelectTrigger>
              <SelectContent side="top">
                <SelectGroup>
                  {[10, 20, 30, 40, 50].map((pageSize) => (
                    <SelectItem key={pageSize} value={`${pageSize}`}>
                      {pageSize}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </div>
          <div className="flex w-fit items-center justify-center text-sm font-medium">
            Page {table.getState().pagination.pageIndex + 1} of{" "}
            {table.getPageCount()}
          </div>
          <div className="ml-auto flex items-center gap-2 lg:ml-0">
            <Button
              variant="outline"
              className="hidden h-8 w-8 p-0 lg:flex"
              onClick={() => table.setPageIndex(0)}
              disabled={!table.getCanPreviousPage()}
            >
              <span className="sr-only">Go to first page</span>
              <ChevronsLeftIcon />
            </Button>
            <Button
              variant="outline"
              className="size-8"
              size="icon"
              onClick={() => table.previousPage()}
              disabled={!table.getCanPreviousPage()}
            >
              <span className="sr-only">Go to previous page</span>
              <ChevronLeftIcon />
            </Button>
            <Button
              variant="outline"
              className="size-8"
              size="icon"
              onClick={() => table.nextPage()}
              disabled={!table.getCanNextPage()}
            >
              <span className="sr-only">Go to next page</span>
              <ChevronRightIcon />
            </Button>
            <Button
              variant="outline"
              className="hidden size-8 lg:flex"
              size="icon"
              onClick={() => table.setPageIndex(table.getPageCount() - 1)}
              disabled={!table.getCanNextPage()}
            >
              <span className="sr-only">Go to last page</span>
              <ChevronsRightIcon />
            </Button>
          </div>
        </div>
      </div>
    </div>
  )
}
