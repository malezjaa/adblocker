import { LogoIcon } from "@/components/logo"
import { Button } from "@/components/ui/button"
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group"
import { cn } from "@/lib/utils"
import { EyeIcon, EyeOffIcon, LogInIcon } from "lucide-react"
import type React from "react"
import { useState } from "react"

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

// eslint-disable-next-line @typescript-eslint/no-unused-vars
async function handleLogin(_password: string): Promise<void> {
  // TODO: implement login logic
}

export function AuthPage() {
  const [password, setPassword] = useState("")
  const [showPassword, setShowPassword] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [isLoading, setIsLoading] = useState(false)

  const validate = (): string | null => {
    if (!password) return "Password is required."
    return null
  }

  const handleSubmit = async (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault()
    setError(null)

    const validationError = validate()
    if (validationError) {
      setError(validationError)
      return
    }

    setIsLoading(true)
    try {
      await handleLogin(password)
    } catch (err) {
      setError("Incorrect password. Please try again.")
    } finally {
      setIsLoading(false)
    }
  }

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

          <form className="space-y-2" onSubmit={handleSubmit} noValidate>
            <div className="space-y-1">
              <InputGroup>
                <InputGroupInput
                  aria-describedby={error ? "password-error" : undefined}
                  aria-invalid={!!error}
                  aria-label="Password"
                  className={cn(
                    !showPassword &&
                      "[letter-spacing:0.2em] placeholder:[letter-spacing:normal]",
                    error && "border-destructive focus-visible:ring-destructive"
                  )}
                  disabled={isLoading}
                  onChange={(e) => {
                    setPassword(e.target.value)
                    if (error) setError(null)
                  }}
                  placeholder="Your password"
                  type={showPassword ? "text" : "password"}
                  value={password}
                />
                <InputGroupAddon align="inline-start">
                  <LogInIcon />
                </InputGroupAddon>
                <InputGroupAddon align="inline-end">
                  <button
                    aria-label={
                      showPassword ? "Hide password" : "Show password"
                    }
                    className="text-muted-foreground transition-colors hover:text-foreground"
                    onClick={() => setShowPassword((prev) => !prev)}
                    type="button"
                  >
                    {showPassword ? (
                      <EyeOffIcon className="h-4 w-4" />
                    ) : (
                      <EyeIcon className="h-4 w-4" />
                    )}
                  </button>
                </InputGroupAddon>
              </InputGroup>

              {error && (
                <p
                  className="text-xs text-destructive"
                  id="password-error"
                  role="alert"
                >
                  {error}
                </p>
              )}
            </div>

            <Button
              className="w-full"
              disabled={isLoading}
              size="sm"
              type="submit"
            >
              {isLoading ? "Logging in…" : "Log In"}
            </Button>
          </form>

          <FullWidthDivider position="bottom" />
        </div>
      </div>
    </div>
  )
}
