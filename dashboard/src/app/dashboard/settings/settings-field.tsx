import { Input } from "@/components/ui/input.tsx"
import { Label } from "@/components/ui/label.tsx"
import { Switch } from "@/components/ui/switch.tsx"
import { Textarea } from "@/components/ui/textarea.tsx"
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select.tsx"
import type { UpstreamServer } from "@/app/dashboard/settings/user-settings.ts"

type SettingValue = string | boolean | UpstreamServer[]

interface BaseSettingDef<TKey extends string> {
  key: TKey
  label: string
  description?: string
}

interface ToggleSettingDef<TKey extends string> extends BaseSettingDef<TKey> {
  type: "toggle"
}

interface TextSettingDef<TKey extends string> extends BaseSettingDef<TKey> {
  type: "text"
  placeholder?: string
}

interface TextareaSettingDef<TKey extends string> extends BaseSettingDef<TKey> {
  type: "textarea"
  placeholder?: string
}

interface SelectSettingDef<TKey extends string> extends BaseSettingDef<TKey> {
  type: "select"
  options: { label: string; value: string }[]
}

export type SettingDef<TKey extends string = string> =
  | ToggleSettingDef<TKey>
  | TextSettingDef<TKey>
  | TextareaSettingDef<TKey>
  | SelectSettingDef<TKey>

interface SettingsFieldProps {
  def: SettingDef
  value: SettingValue
  onChange: (value: SettingValue) => void
  disabled?: boolean
}

export function SettingsField({
  def,
  value,
  onChange,
  disabled,
}: SettingsFieldProps) {
  const inputId = `setting-${def.key}`

  return (
    <div className="flex flex-col gap-3 py-5 first:pt-0 last:pb-0 sm:flex-row sm:items-start sm:justify-between sm:gap-6">
      <div className="space-y-0.5 sm:max-w-sm">
        <Label htmlFor={inputId} className="text-sm font-medium">
          {def.label}
        </Label>
        {def.description ? (
          <p className="text-sm text-muted-foreground">{def.description}</p>
        ) : null}
      </div>

      <div className="w-full sm:w-64 sm:shrink-0">
        {def.type === "toggle" && (
          <div className="flex sm:justify-end">
            <Switch
              id={inputId}
              checked={Boolean(value)}
              onCheckedChange={onChange}
              disabled={disabled}
            />
          </div>
        )}

        {def.type === "text" && (
          <Input
            id={inputId}
            value={String(value ?? "")}
            placeholder={def.placeholder}
            disabled={disabled}
            onChange={(e) => onChange(e.target.value)}
          />
        )}

        {def.type === "textarea" && (
          <Textarea
            id={inputId}
            value={String(value ?? "")}
            placeholder={def.placeholder}
            disabled={disabled}
            onChange={(e) => onChange(e.target.value)}
            className="resize-none"
            rows={3}
          />
        )}

        {def.type === "select" && (
          <Select
            value={String(value ?? "")}
            onValueChange={onChange}
            disabled={disabled}
          >
            <SelectTrigger id={inputId} className="w-full">
              <SelectValue placeholder="Select an option">
                {
                  def.options.find(
                    (option) => option.value === String(value ?? "")
                  )?.label
                }
              </SelectValue>
            </SelectTrigger>
            <SelectContent>
              {def.options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        )}
      </div>
    </div>
  )
}
