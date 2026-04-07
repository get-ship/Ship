import { useMemo } from 'react'
import { Lock, Shield, ShieldCheck, ShieldOff, Wrench } from 'lucide-react'
import type { ProfilePermissions, McpServerConfig } from '@ship/ui'
import { SectionShell } from './SectionShell'
import { PERMISSION_PRESETS } from '#/features/agents/permission-presets'

interface PermissionsSectionProps {
  permissions: ProfilePermissions
  activePreset: string
  onPresetChange: (preset: string) => void
  onEdit?: () => void
  mcpServers?: McpServerConfig[]
}

const PRESET_ICONS: Record<string, React.ReactNode> = {
  'ship-standard': <Shield className="size-3.5" />,
  'ship-autonomous': <ShieldCheck className="size-3.5" />,
  'ship-readonly': <ShieldOff className="size-3.5" />,
}

export function PermissionsSection({
  permissions,
  activePreset,
  onPresetChange,
  onEdit,
}: PermissionsSectionProps) {
  const presets = useMemo(() => PERMISSION_PRESETS, [])

  return (
    <SectionShell
      icon={<Lock className="size-4" />}
      title="Permissions"
      actionLabel="Edit"
      onAction={onEdit}
    >
      {/* Preset buttons */}
      <div className="flex gap-1.5 mb-3">
        {presets.map((preset) => (
          <button
            key={preset.id}
            onClick={() => onPresetChange(preset.id)}
            className={`flex-1 flex items-center gap-1.5 rounded-lg border px-3 py-2 text-left transition-colors ${
              activePreset === preset.id
                ? 'border-primary text-primary bg-primary/5'
                : 'border-border/40 text-muted-foreground/60 bg-card/30 hover:border-border hover:text-muted-foreground'
            }`}
          >
            {PRESET_ICONS[preset.id] ?? <Shield className="size-3.5" />}
            <div className="min-w-0">
              <div className="text-[11px] font-medium">{preset.label}</div>
              <div className="text-[9px] opacity-60 truncate">{preset.description}</div>
            </div>
          </button>
        ))}
      </div>

      {/* Permission summary cards */}
      <div className="grid grid-cols-3 gap-2">
        <PermCard
          icon={<Wrench className="size-3" />}
          label="Allow"
          items={permissions.tools_allow ?? []}
          variant="allow"
        />
        <PermCard
          icon={<Wrench className="size-3" />}
          label="Ask"
          items={permissions.tools_ask ?? []}
          variant="neutral"
        />
        <PermCard
          icon={<Wrench className="size-3" />}
          label="Deny"
          items={permissions.tools_deny ?? []}
          variant="deny"
        />
      </div>
    </SectionShell>
  )
}

function PermCard({
  icon,
  label,
  items,
  variant,
}: {
  icon: React.ReactNode
  label: string
  items: string[]
  variant: 'allow' | 'deny' | 'neutral'
}) {
  const colorCls =
    variant === 'allow' ? 'text-emerald-600 dark:text-emerald-400'
    : variant === 'deny' ? 'text-destructive'
    : 'text-amber-600 dark:text-amber-400'

  return (
    <div className="rounded-lg border border-border/40 bg-card/30 px-3 py-2.5">
      <div className="flex items-center gap-1.5 text-[11px] text-muted-foreground/60 mb-1.5">
        {icon}
        {label}
      </div>
      {items.length > 0 ? (
        <div className={`text-[11px] truncate ${colorCls}`}>
          {items.join(', ')}
        </div>
      ) : (
        <div className="text-[10px] text-muted-foreground/30 italic">none</div>
      )}
    </div>
  )
}
