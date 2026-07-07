import { useEffect, useState } from "react"
import { DashboardPage } from "@/components/app/dashboard-page.tsx"
import {
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card.tsx"
import { DashboardCard } from "@/components/app/dashboard-card.tsx"
import {
  type SettingDef,
  SettingsField,
} from "@/app/dashboard/settings/settings-field.tsx"
import { UpstreamsField } from "@/app/dashboard/settings/upstreams-field.tsx"
import { SettingsToolbar } from "@/app/dashboard/settings/settings-toolbar.tsx"
import {
  type UpstreamServer,
  type UserSettings,
  useUpdateUserSettings,
  useUserSettings,
} from "@/app/dashboard/settings/user-settings.ts"

const dnsFields: SettingDef<keyof UserSettings>[] = [
  {
    key: "dnssec",
    type: "toggle",
    label: "DNSSEC",
    description: "Enabled verifying of DNS records",
  },
]

export default function Settings() {
  const { data: settings, isLoading } = useUserSettings()
  const { mutate: saveSettings, isPending: isSaving } = useUpdateUserSettings()

  const [draft, setDraft] = useState<UserSettings | null>(null)

  useEffect(() => {
    if (settings) setDraft(settings)
  }, [settings])

  const isDirty = Boolean(
    settings && draft && JSON.stringify(settings) !== JSON.stringify(draft)
  )

  function updateField(
    key: keyof UserSettings,
    value: string | boolean | UpstreamServer[]
  ) {
    setDraft((prev) => (prev ? { ...prev, [key]: value } : prev))
  }

  function handleCancel() {
    if (settings) setDraft(settings)
  }

  function handleSave() {
    if (draft) saveSettings(draft)
  }

  return (
    <DashboardPage className="gap-6 pb-24">
      <DashboardCard className="w-full">
        <CardHeader>
          <CardDescription>Configure your DNS settings</CardDescription>
          <CardTitle>DNS</CardTitle>
        </CardHeader>
        <CardContent className="divide-y">
          <UpstreamsField
            label="Upstream DNS servers"
            description="Queried in order. The first one that responds is used."
            value={draft?.upstreams ?? []}
            onChange={(value) => updateField("upstreams", value)}
            disabled={isLoading || !draft}
          />

          {dnsFields.map((def) => (
            <SettingsField
              key={def.key}
              def={def}
              value={draft ? draft[def.key] : ""}
              onChange={(value) => updateField(def.key, value)}
              disabled={isLoading || !draft}
            />
          ))}
        </CardContent>
      </DashboardCard>

      <SettingsToolbar
        open={isDirty}
        isSaving={isSaving}
        onSave={handleSave}
        onCancel={handleCancel}
      />
    </DashboardPage>
  )
}
