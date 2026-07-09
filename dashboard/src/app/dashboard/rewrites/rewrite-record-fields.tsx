import { Input } from "@/components/ui/input.tsx"
import { Label } from "@/components/ui/label.tsx"
import type {
  RewriteRecord,
  RewriteRecordValue,
} from "@/app/dashboard/settings/user-settings.ts"
import { csv, fromCsv } from "./rewrite-utils.ts"
import { LabelWithHelp } from "./rewrite-help.tsx"

type RewriteRecordFieldsProps = {
  record: RewriteRecord
  index: number
  onRecordValueChange: (index: number, value: RewriteRecordValue) => void
}

export function RewriteRecordFields({
  record,
  index,
  onRecordValueChange,
}: RewriteRecordFieldsProps) {
  const value = record.value

  switch (value.type) {
    case "A":
    case "AAAA":
    case "CNAME":
    case "PTR":
      return (
        <div className="grid gap-2">
          <Label htmlFor={`rewrite-record-${index}-value`}>Value</Label>
          <Input
            id={`rewrite-record-${index}-value`}
            value={value.value}
            onChange={(event) =>
              onRecordValueChange(index, {
                ...value,
                value: event.target.value,
              })
            }
            placeholder={value.type === "AAAA" ? "::1" : "target.local."}
          />
        </div>
      )
    case "MX":
      return (
        <div className="grid gap-3 sm:grid-cols-[1fr_120px]">
          <div className="grid gap-2">
            <Label htmlFor={`rewrite-record-${index}-exchange`}>Exchange</Label>
            <Input
              id={`rewrite-record-${index}-exchange`}
              value={value.exchange}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  exchange: event.target.value,
                })
              }
              placeholder="mail.local."
            />
          </div>
          <div className="grid gap-2">
            <LabelWithHelp
              htmlFor={`rewrite-record-${index}-preference`}
              help="Lower MX preference values are tried first."
            >
              Preference
            </LabelWithHelp>
            <Input
              id={`rewrite-record-${index}-preference`}
              type="number"
              min={0}
              value={value.preference}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  preference: Number(event.target.value),
                })
              }
            />
          </div>
        </div>
      )
    case "TXT":
      return (
        <div className="grid gap-2">
          <Label htmlFor={`rewrite-record-${index}-txt`}>Text strings</Label>
          <Input
            id={`rewrite-record-${index}-txt`}
            value={csv(value.value)}
            onChange={(event) =>
              onRecordValueChange(index, {
                ...value,
                value: fromCsv(event.target.value),
              })
            }
            placeholder="v=spf1 -all, verification-token"
          />
        </div>
      )
    case "SRV":
      return (
        <div className="grid gap-3 sm:grid-cols-[1fr_repeat(3,100px)]">
          <div className="grid gap-2">
            <Label htmlFor={`rewrite-record-${index}-target`}>Target</Label>
            <Input
              id={`rewrite-record-${index}-target`}
              value={value.target}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  target: event.target.value,
                })
              }
              placeholder="svc.local."
            />
          </div>
          <div className="grid gap-2">
            <LabelWithHelp
              htmlFor={`rewrite-record-${index}-priority`}
              help="Lower SRV priority values are preferred first."
            >
              Priority
            </LabelWithHelp>
            <Input
              id={`rewrite-record-${index}-priority`}
              type="number"
              min={0}
              value={value.priority}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  priority: Number(event.target.value),
                })
              }
            />
          </div>
          <div className="grid gap-2">
            <LabelWithHelp
              htmlFor={`rewrite-record-${index}-weight`}
              help="Weight balances traffic between SRV records with the same priority."
            >
              Weight
            </LabelWithHelp>
            <Input
              id={`rewrite-record-${index}-weight`}
              type="number"
              min={0}
              value={value.weight}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  weight: Number(event.target.value),
                })
              }
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor={`rewrite-record-${index}-port`}>Port</Label>
            <Input
              id={`rewrite-record-${index}-port`}
              type="number"
              min={0}
              max={65535}
              value={value.port}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  port: Number(event.target.value),
                })
              }
            />
          </div>
        </div>
      )
    case "HTTPS":
    case "SVCB":
      return (
        <div className="grid gap-3 sm:grid-cols-[120px_1fr]">
          <div className="grid gap-2">
            <LabelWithHelp
              htmlFor={`rewrite-record-${index}-svcb-priority`}
              help="Priority 0 is alias mode; larger values describe service endpoints."
            >
              Priority
            </LabelWithHelp>
            <Input
              id={`rewrite-record-${index}-svcb-priority`}
              type="number"
              min={0}
              value={value.priority}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  priority: Number(event.target.value),
                })
              }
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor={`rewrite-record-${index}-svcb-target`}>
              Target
            </Label>
            <Input
              id={`rewrite-record-${index}-svcb-target`}
              value={value.target}
              onChange={(event) =>
                onRecordValueChange(index, {
                  ...value,
                  target: event.target.value,
                })
              }
              placeholder="."
            />
          </div>
        </div>
      )
  }
}
