import {
  Terminal, Info, ExternalLink, Cpu, Settings2, Check, Loader2,
} from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import {
  SettingsSection, SettingsRow, SettingsSelect, SettingsInput,
  CollapsibleSection,
} from './SettingsLayout'
import { useSettings, type ProviderStatus } from './useSettings'

// ── Inline feedback badge ──────────────────────────────────────────────────

function SaveBadge({ settingKey, saving, saved, error }: {
  settingKey: string
  saving: string | null
  saved: string | null
  error: string | null
}) {
  if (saving === settingKey) {
    return <Loader2 className="ml-2 size-3 animate-spin text-muted-foreground" />
  }
  if (saved === settingKey) {
    return <Check className="ml-2 size-3 text-emerald-500" />
  }
  if (error) {
    return <span className="ml-2 text-[10px] text-destructive">{error}</span>
  }
  return null
}

// ── Providers ──────────────────────────────────────────────────────────────

export function ProvidersSection() {
  const { settings, providers, updateSetting, saving, saved, error } = useSettings()
  const data = settings.data ?? {}
  const providerList = providers.data ?? []
  const available = providerList.filter((p) => p.available)

  const providerOptions = available.map((p) => ({ value: p.id, label: p.id }))
  const currentProvider = data['defaults.provider'] ?? ''
  const currentModel = data['defaults.model'] ?? ''

  return (
    <SettingsSection icon={<Cpu className="size-[15px]" />} title="Providers">
      <SettingsRow label="Default provider" sublabel="Used when no provider is specified">
        <div className="flex items-center">
          {providerOptions.length > 0 ? (
            <SettingsSelect
              value={currentProvider}
              onChange={(v) => updateSetting('defaults.provider', v)}
              options={providerOptions}
            />
          ) : (
            <span className="text-[11px] text-muted-foreground">No providers available</span>
          )}
          <SaveBadge settingKey="defaults.provider" saving={saving} saved={saved} error={null} />
        </div>
      </SettingsRow>

      <SettingsRow label="Default model" sublabel="Model name passed to the provider">
        <div className="flex items-center">
          <SettingsInput
            value={currentModel}
            onSave={(v) => updateSetting('defaults.model', v)}
            placeholder="e.g. sonnet"
          />
          <SaveBadge settingKey="defaults.model" saving={saving} saved={saved} error={null} />
        </div>
      </SettingsRow>

      {providerList.length > 0 && (
        <div className="mt-2 border-t border-border/30 pt-2">
          <div className="mb-1.5 text-[10px] font-semibold uppercase tracking-wider text-muted-foreground">
            Installed providers
          </div>
          {providerList.map((p) => (
            <ProviderRow key={p.id} provider={p} />
          ))}
        </div>
      )}

      {error && (
        <div className="mt-2 text-[11px] text-destructive">{error}</div>
      )}
    </SettingsSection>
  )
}

function ProviderRow({ provider }: { provider: ProviderStatus }) {
  return (
    <div className="flex items-center justify-between py-1">
      <div className="flex items-center gap-2">
        <span
          className={`size-2 rounded-full ${provider.available ? 'bg-emerald-500' : 'bg-muted-foreground/40'}`}
        />
        <span className="text-[11px] text-foreground/80 font-mono">{provider.id}</span>
      </div>
      <div className="flex items-center gap-2 text-[10px] text-muted-foreground">
        <span>{provider.available ? 'installed' : 'not found'}</span>
        {provider.version && (
          <span className="font-mono">v{provider.version}</span>
        )}
      </div>
    </div>
  )
}

// ── Terminal ───────────────────────────────────────────────────────────────

export function TerminalSection() {
  const { settings, updateSetting, saving, saved, error } = useSettings()
  const data = settings.data ?? {}

  const termProgram = data['terminal.program'] ?? ''
  const worktreeBase = data['worktree.base_dir'] ?? ''

  const terminalOptions = [
    { value: 'tmux', label: 'tmux' },
    { value: 'iterm', label: 'iTerm' },
    { value: 'wt', label: 'Windows Terminal' },
    { value: 'gnome', label: 'GNOME Terminal' },
    { value: 'vscode', label: 'VS Code' },
  ]

  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="Terminal">
      <SettingsRow label="Terminal program" sublabel="Used when launching sessions">
        <div className="flex items-center">
          <SettingsSelect
            value={termProgram}
            onChange={(v) => updateSetting('terminal.program', v)}
            options={terminalOptions}
          />
          <SaveBadge settingKey="terminal.program" saving={saving} saved={saved} error={null} />
        </div>
      </SettingsRow>

      <SettingsRow label="Worktree base directory" sublabel="Root for git worktrees">
        <div className="flex items-center">
          <SettingsInput
            value={worktreeBase}
            onSave={(v) => updateSetting('worktree.base_dir', v)}
            placeholder="/path/to/worktrees"
          />
          <SaveBadge settingKey="worktree.base_dir" saving={saving} saved={saved} error={null} />
        </div>
      </SettingsRow>

      {error && (
        <div className="mt-2 text-[11px] text-destructive">{error}</div>
      )}
    </SettingsSection>
  )
}

// ── Advanced (collapsed by default) ────────────────────────────────────────

export function AdvancedSection() {
  const { connected, error: daemonError } = useDaemon()
  const { settings, updateSetting, saving, saved, error } = useSettings()
  const data = settings.data ?? {}

  const registryEndpoint = data['registry.endpoint'] ?? ''

  return (
    <CollapsibleSection
      icon={<Settings2 className="size-[15px]" />}
      title="Advanced"
    >
      <SettingsRow label="Daemon status">
        <div className="flex items-center gap-1.5">
          <span className={`size-2 rounded-full ${connected ? 'bg-emerald-500' : 'bg-muted-foreground'}`} />
          <span className="text-[11px] text-muted-foreground">
            {connected ? 'Connected' : 'Disconnected'}
          </span>
        </div>
      </SettingsRow>

      <SettingsRow label="Daemon endpoint">
        <span className="text-[11px] text-muted-foreground font-mono">{DAEMON_BASE_URL}</span>
      </SettingsRow>

      <SettingsRow label="Registry endpoint" sublabel="Package registry URL">
        <div className="flex items-center">
          <SettingsInput
            value={registryEndpoint}
            onSave={(v) => updateSetting('registry.endpoint', v)}
            placeholder="https://registry.getship.dev"
          />
          <SaveBadge settingKey="registry.endpoint" saving={saving} saved={saved} error={null} />
        </div>
      </SettingsRow>

      {(daemonError || error) && (
        <div className="mt-2 text-[11px] text-destructive">
          {daemonError?.message ?? error}
        </div>
      )}
    </CollapsibleSection>
  )
}

// ── About ──────────────────────────────────────────────────────────────────

export function AboutSection() {
  return (
    <SettingsSection icon={<Info className="size-[15px]" />} title="About">
      <SettingsRow label="Documentation">
        <a
          href="https://getship.dev/docs"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-1 text-[11px] text-primary transition hover:underline"
        >
          getship.dev/docs
          <ExternalLink className="size-3" />
        </a>
      </SettingsRow>
      <SettingsRow label="Source">
        <a
          href="https://github.com/madvib/Ship"
          target="_blank"
          rel="noopener noreferrer"
          className="flex items-center gap-1 text-[11px] text-primary transition hover:underline"
        >
          GitHub
          <ExternalLink className="size-3" />
        </a>
      </SettingsRow>
    </SettingsSection>
  )
}
