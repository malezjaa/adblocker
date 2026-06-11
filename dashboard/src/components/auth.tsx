import { Logo, LogoIcon } from "@/components/logo"
import { Button } from "@/components/ui/button"
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group"
import { cn } from "@/lib/utils"
import { LogInIcon } from "lucide-react"
import type React from "react"

export function AuthDivider({
  children,
  ...props
}: React.ComponentProps<"div">) {
  return (
    <div className="relative flex w-full items-center" {...props}>
      <div className="w-full border-t" />
      <div className="flex w-max justify-center px-2 text-xs text-nowrap text-muted-foreground">
        {children}
      </div>
      <div className="w-full border-t" />
    </div>
  )
}

type FullWidthDividerProps = React.ComponentProps<"div"> & {
  contained?: boolean
  position?: "top" | "bottom"
}

export function FullWidthDivider({
  className,
  contained = false,
  position,
  ...props
}: FullWidthDividerProps) {
  return (
    <div
      aria-hidden="true"
      className={cn(
        "pointer-events-none absolute h-px bg-border",
        "data-[contained=false]:left-1/2 data-[contained=false]:w-screen data-[contained=false]:-translate-x-1/2",
        "data-[contained=true]:inset-x-0 data-[contained=true]:w-full",
        position &&
          "data-[position=bottom]:-bottom-px data-[position=top]:-top-px",
        className
      )}
      data-contained={contained}
      data-position={position}
      {...props}
    />
  )
}

export function AuthPage() {
  return (
    <div className="relative w-full overflow-hidden px-4 md:h-screen">
      <div className="relative mx-auto flex min-h-screen w-full max-w-sm flex-col justify-center border-x *:px-6">
        <div className="flex flex-col space-y-6">
          <a aria-label="Home" className="flex flex-row gap-3" href="#">
            <LogoIcon className="h-4.5" />
            DNS
          </a>
          <div className="space-y-1">
            <h1 className="text-xl font-semibold tracking-wide">
              Hey, welcome!
            </h1>
            <p className="text-base text-muted-foreground">
              Log in to access the dashboard.
            </p>
          </div>
        </div>

        <div className="relative my-6 flex size-full flex-col gap-4 py-8">
          <FullWidthDivider position="top" />

          <form className="space-y-2">
            <InputGroup>
              <InputGroupInput
                aria-label="Password"
                placeholder="Your password"
                type="password"
              />
              <InputGroupAddon align="inline-start">
                <LogInIcon />
              </InputGroupAddon>
            </InputGroup>

            <Button className="w-full" size="sm" type="submit">
              Log In
            </Button>
          </form>
          <FullWidthDivider position="bottom" />
        </div>
      </div>
    </div>
  )
}
