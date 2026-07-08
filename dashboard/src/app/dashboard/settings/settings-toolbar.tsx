import { Button } from "@/components/ui/button.tsx"
import { Loader2 } from "lucide-react"
import { cn } from "@/lib/utils.ts"

interface SettingsToolbarProps {
  open: boolean
  isSaving?: boolean
  saveDisabled?: boolean
  onSave: () => void
  onCancel: () => void
}

export function SettingsToolbar({
  open,
  isSaving,
  saveDisabled,
  onSave,
  onCancel,
}: SettingsToolbarProps) {
  return (
    <div
      aria-hidden={!open}
      className={cn(
        "pointer-events-none fixed inset-x-0 bottom-6 z-50 flex justify-center px-4 transition-all duration-300 ease-out",
        open ? "translate-y-0 opacity-100" : "translate-y-4 opacity-0"
      )}
    >
      <div
        className={cn(
          "flex items-center gap-4 rounded-none border bg-background/95 px-4 py-2 shadow-lg backdrop-blur-sm",
          open && "pointer-events-auto"
        )}
      >
        <span className="pl-2 text-sm text-muted-foreground">
          You have unsaved changes
        </span>
        <div className="flex items-center gap-2">
          <Button
            variant="ghost"
            size="sm"
            onClick={onCancel}
            disabled={isSaving}
          >
            Cancel
          </Button>
          <Button
            size="sm"
            onClick={onSave}
            disabled={isSaving || saveDisabled}
          >
            {isSaving ? (
              <>
                <Loader2 className="size-4 animate-spin" />
                Saving
              </>
            ) : (
              "Save changes"
            )}
          </Button>
        </div>
      </div>
    </div>
  )
}
