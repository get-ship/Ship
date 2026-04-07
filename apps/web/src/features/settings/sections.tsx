import {
  Terminal, Info,
  ExternalLink, Activity,
} from 'lucide-react'
import { useDaemon } from '#/features/studio/hooks/useDaemonData'
import { DAEMON_BASE_URL } from '#/lib/daemon-config'
import { SettingsSection, SettingsRow } from './SettingsLayout'

// ── Daemon Connection ───────────────────────────────────────────────────────

export function CLIConnectionSection() {
  const { connected, workspaces, jobs, error } = useDaemon()
  const activeWorkspace = workspaces.find((w) => w.status === 'active')
  const runningJobs = jobs.filter((j) => j.status === 'dispatched').length

  return (
    <SettingsSection icon={<Terminal className="size-[15px]" />} title="Daemon">
      <SettingsRow label="Status">
        <div className="flex items-center gap-1.5">
          <span className={`size-2 rounded-full ${connected ? 'bg-emerald-500' : 'bg-muted-foreground'}`} />
          <span className="text-[11px] text-muted-foreground">{connected ? 'Connected' : 'Disconnected'}</span>
        </div>
      </SettingsRow>

      <SettingsRow label="Endpoint">
        <span className="text-[11px] text-muted-foreground font-mono">{DAEMON_BASE_URL}</span>
      </SettingsRow>

      <SettingsRow label="Workspaces">
        <span className="text-[11px] text-muted-foreground">{workspaces.length}</span>
      </SettingsRow>

      {activeWorkspace && (
        <SettingsRow label="Active workspace">
          <span className="text-[11px] text-muted-foreground font-mono truncate max-w-[200px]">{activeWorkspace.branch}</span>
        </SettingsRow>
      )}

      {activeWorkspace?.active_agent && (
        <SettingsRow label="Active agent">
          <span className="text-[11px] text-muted-foreground">{activeWorkspace.active_agent}</span>
        </SettingsRow>
      )}

      <SettingsRow label="Running jobs">
        <div className="flex items-center gap-1.5">
          {runningJobs > 0 && <span className="size-1.5 rounded-full bg-sky-500 animate-pulse" />}
          <span className="text-[11px] text-muted-foreground">{runningJobs}</span>
        </div>
      </SettingsRow>

      {error && (
        <SettingsRow label="Error">
          <span className="max-w-[300px] text-right text-[11px] text-destructive">{error.message}</span>
        </SettingsRow>
      )}
    </SettingsSection>
  )
}

// ── Integrations ─────────────────────────────────────────────────────────────

export function IntegrationsSection() {
  return (
    <SettingsSection icon={<Activity className="size-[15px]" />} title="Integrations">
      <SettingsRow label="Providers" sublabel="Manage via ship use <provider>">
        <span className="text-[11px] text-muted-foreground font-mono">ship use</span>
      </SettingsRow>
      <SettingsRow label="Worktree directory" sublabel="Override via ship config">
        <span className="text-[11px] text-muted-foreground font-mono">ship config</span>
      </SettingsRow>
    </SettingsSection>
  )
}

// ── About ────────────────────────────────────────────────────────────────────

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
