import { useState } from 'react'
import { SlidersHorizontal } from 'lucide-react'
import type { HookConfig } from '@ship/set'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { useProviderData, type ProviderStatus } from '#/features/agents/useProviderData'
import { SectionShell } from './SectionShell'
import { ProviderConfigModal } from '../dialogs/ProviderConfigModal'

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude',
  gemini: 'Gemini',
  codex: 'Codex',
  cursor: 'Cursor',
  opencode: 'OpenCode',
}

export interface ProvidersSectionProps {
  providers: string[]
  model?: string | null
  hooks: HookConfig[]
  providerSettings: Record<string, Record<string, unknown>>
  onChangeModel: (model: string | null) => void
  onChangeHooks: (hooks: HookConfig[]) => void
  onChangeProviderSettings: (settings: Record<string, Record<string, unknown>>) => void
}

export function ProvidersSection({
  providers,
  model,
  hooks,
  providerSettings,
  onChangeModel,
  onChangeHooks,
  onChangeProviderSettings,
}: ProvidersSectionProps) {
  const [activeProvider, setActiveProvider] = useState<string | null>(null)
  const { providers: apiProviders, defaultProvider, defaultModel } = useProviderData()

  if (providers.length === 0) return null

  const statusMap = new Map(apiProviders.map((p) => [p.id, p]))

  const handleSave = (config: {
    model: string | null
    hooks: HookConfig[]
    providerSettings: Record<string, unknown>
  }) => {
    onChangeModel(config.model)
    onChangeHooks(config.hooks)
    if (activeProvider) {
      onChangeProviderSettings({ ...providerSettings, [activeProvider]: config.providerSettings })
    }
  }

  return (
    <>
      <SectionShell
        icon={<SlidersHorizontal className="size-4" />}
        title="Providers"
        count={`${providers.length} provider${providers.length !== 1 ? 's' : ''}`}
      >
        <div className="grid grid-cols-1 sm:grid-cols-2 gap-2">
          {providers.map((provider) => (
            <ProviderCard
              key={provider}
              provider={provider}
              model={model}
              hookCount={hooks.length}
              settingsCount={Object.keys(providerSettings[provider] ?? {}).length}
              status={statusMap.get(provider)}
              isDefault={provider === defaultProvider}
              onClick={() => setActiveProvider(provider)}
            />
          ))}
        </div>
      </SectionShell>

      {activeProvider && (
        <ProviderConfigModal
          open={activeProvider !== null}
          onOpenChange={(open) => { if (!open) setActiveProvider(null) }}
          provider={activeProvider}
          model={model ?? null}
          hooks={hooks}
          providerSettings={providerSettings[activeProvider] ?? {}}
          defaultModel={defaultModel}
          onSave={handleSave}
        />
      )}
    </>
  )
}

function ProviderCard({
  provider,
  model,
  hookCount,
  settingsCount,
  status,
  isDefault,
  onClick,
}: {
  provider: string
  model?: string | null
  hookCount: number
  settingsCount: number
  status?: ProviderStatus
  isDefault: boolean
  onClick: () => void
}) {
  const parts: string[] = []
  if (model) parts.push(`model: ${model}`)
  if (hookCount > 0) parts.push(`${hookCount} hook${hookCount !== 1 ? 's' : ''}`)
  if (settingsCount > 0) parts.push(`${settingsCount} custom setting${settingsCount !== 1 ? 's' : ''}`)
  const summary = parts.length > 0 ? parts.join(', ') : 'defaults'

  return (
    <button
      onClick={onClick}
      className="flex items-center gap-3 rounded-lg border border-border/40 bg-card/30 px-3 py-3 text-left hover:border-border transition-colors"
    >
      <ProviderLogo provider={provider} size="md" />
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-1.5">
          <span className="text-xs font-medium text-foreground/80">
            {PROVIDER_LABELS[provider] ?? provider}
          </span>
          {isDefault && (
            <span className="rounded bg-primary/10 px-1.5 py-0.5 text-[9px] font-medium text-primary">
              default
            </span>
          )}
        </div>
        <div className="flex items-center gap-1.5 mt-0.5">
          <AvailabilityDot status={status} />
          <span className="text-[10px] text-muted-foreground/50 truncate">
            {summary}
          </span>
        </div>
      </div>
    </button>
  )
}

function AvailabilityDot({ status }: { status?: ProviderStatus }) {
  if (!status) return null

  if (status.available) {
    return (
      <span className="flex items-center gap-1 shrink-0">
        <span className="size-1.5 rounded-full bg-emerald-500" />
        {status.version && (
          <span className="text-[9px] text-muted-foreground/40">{status.version}</span>
        )}
      </span>
    )
  }

  return (
    <span className="flex items-center gap-1 shrink-0">
      <span className="size-1.5 rounded-full bg-muted-foreground/30" />
      <span className="text-[9px] text-muted-foreground/30">not installed</span>
    </span>
  )
}
