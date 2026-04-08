import { useState, useEffect, useCallback } from 'react'
import { X } from 'lucide-react'
import type { HookConfig } from '@ship/ui'
import { ProviderLogo } from '#/features/compiler/ProviderLogo'
import { ProviderGeneralTab } from './ProviderGeneralTab'
import { ProviderHooksTab } from './ProviderHooksTab'
import { ProviderPassthroughTab } from './ProviderPassthroughTab'

const HOOK_PROVIDERS = new Set(['claude', 'gemini'])

const PROVIDER_LABELS: Record<string, string> = {
  claude: 'Claude', gemini: 'Gemini', codex: 'Codex', cursor: 'Cursor', opencode: 'OpenCode',
}

type TabId = 'general' | 'hooks' | 'passthrough'

export interface ProviderConfigModalProps {
  open: boolean
  onOpenChange: (open: boolean) => void
  provider: string
  model: string | null
  hooks: HookConfig[]
  providerSettings: Record<string, unknown>
  onSave: (config: {
    model: string | null
    hooks: HookConfig[]
    providerSettings: Record<string, unknown>
  }) => void
}

export function ProviderConfigModal({
  open,
  onOpenChange,
  provider,
  model,
  hooks,
  providerSettings,
  onSave,
}: ProviderConfigModalProps) {
  const [activeTab, setActiveTab] = useState<TabId>('general')
  const [localModel, setLocalModel] = useState<string>(model ?? '')
  const [localHooks, setLocalHooks] = useState<HookConfig[]>(hooks)
  const [localSettings, setLocalSettings] = useState<Record<string, unknown>>(providerSettings)

  const supportsHooks = HOOK_PROVIDERS.has(provider)

  useEffect(() => {
    if (open) {
      setActiveTab('general')
      setLocalModel(model ?? '')
      setLocalHooks([...hooks])
      setLocalSettings({ ...providerSettings })
    }
  }, [open, model, hooks, providerSettings])

  const close = useCallback(() => onOpenChange(false), [onOpenChange])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close() }
    document.addEventListener('keydown', onKey)
    return () => document.removeEventListener('keydown', onKey)
  }, [open, close])

  if (!open) return null

  const handleSave = () => {
    onSave({
      model: localModel.trim() || null,
      hooks: localHooks,
      providerSettings: localSettings,
    })
    close()
  }

  const tabs: { id: TabId; label: string }[] = [
    { id: 'general', label: 'General' },
    ...(supportsHooks ? [{ id: 'hooks' as const, label: 'Hooks' }] : []),
    { id: 'passthrough', label: 'Passthrough' },
  ]

  return (
    <>
      <div className="fixed inset-0 z-50 bg-black/50 backdrop-blur-sm" onClick={close} />
      <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
        <div
          role="dialog"
          aria-modal="true"
          className="w-full max-w-2xl rounded-xl border border-border/60 bg-card shadow-2xl flex flex-col max-h-[85vh]"
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center justify-between border-b border-border/60 px-5 py-3.5">
            <div className="flex items-center gap-2">
              <ProviderLogo provider={provider} size="md" />
              <h2 className="font-display text-sm font-semibold text-foreground">
                {PROVIDER_LABELS[provider] ?? provider} Configuration
              </h2>
            </div>
            <button onClick={close} aria-label="Close" className="rounded-md p-1 text-muted-foreground hover:bg-muted hover:text-foreground transition">
              <X className="size-4" />
            </button>
          </div>

          {/* Tabs */}
          <div className="flex border-b border-border/40 px-5">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`px-3 py-2 text-xs font-medium border-b-2 transition-colors -mb-px ${
                  activeTab === tab.id
                    ? 'border-primary text-primary'
                    : 'border-transparent text-muted-foreground/60 hover:text-muted-foreground'
                }`}
              >
                {tab.label}
              </button>
            ))}
          </div>

          {/* Tab content */}
          <div className="flex-1 overflow-y-auto px-5 py-4">
            {activeTab === 'general' && (
              <ProviderGeneralTab
                provider={provider}
                model={localModel}
                onModelChange={setLocalModel}
              />
            )}
            {activeTab === 'hooks' && supportsHooks && (
              <ProviderHooksTab hooks={localHooks} onChange={setLocalHooks} />
            )}
            {activeTab === 'passthrough' && (
              <ProviderPassthroughTab
                provider={provider}
                settings={localSettings}
                onChange={setLocalSettings}
              />
            )}
          </div>

          {/* Footer */}
          <div className="flex items-center justify-end gap-2 border-t border-border/60 px-5 py-3.5">
            <button onClick={close} className="rounded-lg border border-border/60 bg-card px-4 py-2 text-xs font-medium text-muted-foreground transition hover:border-border hover:text-foreground">
              Cancel
            </button>
            <button onClick={handleSave} className="rounded-lg bg-primary px-4 py-2 text-xs font-medium text-primary-foreground transition hover:opacity-90">
              Save
            </button>
          </div>
        </div>
      </div>
    </>
  )
}
