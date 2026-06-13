"use client"

import {Avatar, AvatarFallback} from "@/components/ui/avatar"
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu"
import {LogOutIcon,} from "lucide-react"
import {authLogout} from "@/lib/auth.ts";

export function NavUser() {
  return (
    <DropdownMenu>
      <DropdownMenuTrigger render={<Avatar className="size-8" />}>
        <AvatarFallback>A</AvatarFallback>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-60">
        <div className="flex items-center gap-3 px-2 py-2">
          <Avatar className="size-10">
            <AvatarFallback>A</AvatarFallback>
          </Avatar>

          <div>
            <span className="font-medium text-foreground">Admin</span>
          </div>
        </div>

        <DropdownMenuSeparator />

        <DropdownMenuGroup>
          <DropdownMenuItem
            className="w-full cursor-pointer"
            variant="destructive"
            onClick={async () => {
              await authLogout()
            }}
          >
            <LogOutIcon />
            Log out
          </DropdownMenuItem>
        </DropdownMenuGroup>
      </DropdownMenuContent>
    </DropdownMenu>
  )
}
