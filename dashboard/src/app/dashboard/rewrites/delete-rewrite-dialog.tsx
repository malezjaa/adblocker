import { Loader2 } from "lucide-react"
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog.tsx"
import { Kbd } from "@/components/ui/kbd.tsx"
import type { RewriteEntry } from "@/app/dashboard/settings/user-settings.ts"
import { entryLabel } from "./rewrite-utils.ts"

type DeleteRewriteDialogProps = {
  target: RewriteEntry | null
  isDeleting: boolean
  onOpenChange: (open: boolean) => void
  onConfirm: () => void
}

export function DeleteRewriteDialog({
  target,
  isDeleting,
  onOpenChange,
  onConfirm,
}: DeleteRewriteDialogProps) {
  return (
    <AlertDialog open={target !== null} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>Delete rewrite?</AlertDialogTitle>
          <AlertDialogDescription>
            This will remove <Kbd>{target ? entryLabel(target) : ""}</Kbd>. This
            action can't be undone.
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction
            onClick={onConfirm}
            disabled={isDeleting}
            variant="destructive"
          >
            {isDeleting && <Loader2 className="size-4 animate-spin" />}
            Delete
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  )
}
