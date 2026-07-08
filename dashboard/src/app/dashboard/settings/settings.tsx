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
import {
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@/components/ui/tabs.tsx"

type SettingsValue =
  | string
  | number
  | boolean
  | UpstreamServer[]
  | null
  | undefined

type SettingsFieldDef = SettingDef<string> & {
  path: string
  emptyAsNull?: boolean
}

const serviceFields: SettingsFieldDef[] = [
  {
    key: "dns.enabled",
    path: "dns.enabled",
    type: "toggle",
    label: "Plain DNS",
    description: "Accept standard DNS requests from clients on your network.",
  },
  {
    key: "dns.port",
    path: "dns.port",
    type: "number",
    label: "Plain DNS port",
    description: "UDP port used by the plain DNS listener.",
    min: 1,
    max: 65535,
  },
  {
    key: "doh.enabled",
    path: "doh.enabled",
    type: "toggle",
    label: "DNS-over-HTTPS",
    description: "Accept encrypted DNS queries over HTTPS.",
  },
  {
    key: "doh.port",
    path: "doh.port",
    type: "number",
    label: "DNS-over-HTTPS port",
    description: "TCP port used by the DoH listener.",
    min: 1,
    max: 65535,
  },
  {
    key: "firewall.open_ports",
    path: "firewall.open_ports",
    type: "toggle",
    label: "Open firewall ports",
    description: "Let vox create Windows firewall rules for DNS and DoH.",
  },
]

const resolverFields: SettingsFieldDef[] = [
  {
    key: "resolver.dnssec",
    path: "resolver.dnssec",
    type: "toggle",
    label: "DNSSEC",
    description:
      "Validate upstream DNS responses when supported by the resolver.",
  },
]

const certificateStrategyField: SettingsFieldDef = {
  key: "certs.strategy",
  path: "certs.strategy",
  type: "select",
  label: "Certificate strategy",
  description: "Choose how vox obtains the certificate used by DNS-over-HTTPS.",
  options: [
    { label: "ACME", value: "acme" },
    { label: "Self-signed", value: "self-signed" },
    { label: "Manual", value: "manual" },
    { label: "None", value: "none" },
  ],
}

const acmeFields: SettingsFieldDef[] = [
  {
    key: "certs.acme.directory_url",
    path: "certs.acme.directory_url",
    type: "text",
    label: "ACME directory URL",
    description: "ACME server endpoint used for certificate orders.",
  },
  {
    key: "certs.acme.email",
    path: "certs.acme.email",
    type: "text",
    label: "ACME email",
    description: "Contact email registered with the certificate authority.",
    emptyAsNull: true,
  },
  {
    key: "certs.acme.challenge",
    path: "certs.acme.challenge",
    type: "select",
    label: "ACME challenge",
    description: "Validation method used to prove control of the domain.",
    options: [
      { label: "DNS-01", value: "dns-01" },
      { label: "HTTP-01", value: "http-01" },
      { label: "TLS-ALPN-01", value: "tls-alpn-01" },
    ],
  },
  {
    key: "certs.acme.domain",
    path: "certs.acme.domain",
    type: "text",
    label: "ACME domain",
    description:
      "Domain name that should be present on the issued certificate.",
    emptyAsNull: true,
  },
]

const manualCertificateFields: SettingsFieldDef[] = [
  {
    key: "certs.manual.cert_path",
    path: "certs.manual.cert_path",
    type: "text",
    label: "Certificate path",
    description: "Path to the PEM certificate chain used by the DoH server.",
    emptyAsNull: true,
  },
  {
    key: "certs.manual.key_path",
    path: "certs.manual.key_path",
    type: "text",
    label: "Private key path",
    description: "Path to the PEM private key matching the certificate.",
    emptyAsNull: true,
  },
]

function cloneSettings(settings: UserSettings): UserSettings {
  return JSON.parse(JSON.stringify(settings)) as UserSettings
}

function getPath(source: UserSettings, path: string): unknown {
  return path.split(".").reduce<unknown>((value, key) => {
    if (value && typeof value === "object") {
      return (value as Record<string, unknown>)[key]
    }

    return undefined
  }, source)
}

function setPath(
  source: UserSettings,
  path: string,
  value: unknown
): UserSettings {
  const next = cloneSettings(source)
  const parts = path.split(".")
  let cursor: Record<string, unknown> = next as unknown as Record<
    string,
    unknown
  >

  for (const part of parts.slice(0, -1)) {
    const current = cursor[part]
    const child =
      current && typeof current === "object"
        ? Array.isArray(current)
          ? [...current]
          : { ...(current as Record<string, unknown>) }
        : {}

    cursor[part] = child
    cursor = child as Record<string, unknown>
  }

  cursor[parts[parts.length - 1]] = value
  return next
}

function FieldsGroup({
  fields,
  draft,
  disabled,
  onChange,
}: {
  fields: SettingsFieldDef[]
  draft: UserSettings
  disabled?: boolean
  onChange: (field: SettingsFieldDef, value: SettingsValue) => void
}) {
  return (
    <>
      {fields.map((field) => (
        <SettingsField
          key={field.key}
          def={field}
          value={getPath(draft, field.path) as SettingsValue}
          onChange={(value) => onChange(field, value as SettingsValue)}
          disabled={disabled}
        />
      ))}
    </>
  )
}

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

  function updatePath(field: SettingsFieldDef, value: SettingsValue) {
    const nextValue =
      field.emptyAsNull && typeof value === "string" && value.trim() === ""
        ? null
        : value

    setDraft((prev) => (prev ? setPath(prev, field.path, nextValue) : prev))
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
          <CardDescription>Configure your daemon</CardDescription>
          <CardTitle>Settings</CardTitle>
        </CardHeader>
        <CardContent>
          {draft ? (
            <Tabs defaultValue="services" className="gap-5">
              <TabsList
                variant="line"
                className="w-full justify-start overflow-x-auto"
              >
                <TabsTrigger value="services">Services</TabsTrigger>
                <TabsTrigger value="resolver">Resolver</TabsTrigger>
                <TabsTrigger value="certificates">Certificates</TabsTrigger>
              </TabsList>

              <TabsContent value="services" className="divide-y">
                <FieldsGroup
                  fields={serviceFields}
                  draft={draft}
                  disabled={isLoading || !draft}
                  onChange={updatePath}
                />
              </TabsContent>

              <TabsContent value="resolver" className="divide-y">
                <UpstreamsField
                  label="Upstream DNS servers"
                  description="Providers queried by the resolver in order of preference."
                  value={draft.resolver.upstreams}
                  onChange={(value) =>
                    setDraft((prev) =>
                      prev ? setPath(prev, "resolver.upstreams", value) : prev
                    )
                  }
                  disabled={isLoading || !draft}
                />
                <FieldsGroup
                  fields={resolverFields}
                  draft={draft}
                  disabled={isLoading || !draft}
                  onChange={updatePath}
                />
              </TabsContent>

              <TabsContent value="certificates" className="divide-y">
                <FieldsGroup
                  fields={[certificateStrategyField]}
                  draft={draft}
                  disabled={isLoading || !draft}
                  onChange={updatePath}
                />

                {draft.certs.strategy === "acme" && (
                  <FieldsGroup
                    fields={acmeFields}
                    draft={draft}
                    disabled={isLoading || !draft}
                    onChange={updatePath}
                  />
                )}

                {draft.certs.strategy === "manual" && (
                  <FieldsGroup
                    fields={manualCertificateFields}
                    draft={draft}
                    disabled={isLoading || !draft}
                    onChange={updatePath}
                  />
                )}
              </TabsContent>
            </Tabs>
          ) : null}
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
