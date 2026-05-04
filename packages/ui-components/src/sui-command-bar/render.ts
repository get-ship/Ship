// Rendering helpers for <sui-command-bar>.
// Separated from the controller to stay under the 300-line file cap.

export type BarMode = 'ai' | 'command' | 'find' | 'mention' | 'create'

export type CreateSurface = 'view' | 'skill' | 'agent' | 'pipeline' | 'trigger' | 'theme'

export interface CreateSurfaceDef {
  id: CreateSurface
  label: string
  icon: string
  shapes: string[]
  shapeLabel: string
}

export interface SessionSummary {
  id: string
  workspace_id: string
  agent_id?: string
  provider?: string
  status: string
}

export interface AgentSummary {
  id: string
  name?: string
  label?: string
}

export interface ThreadLine {
  id: string
  sender: 'user' | 'agent' | 'system'
  text: string
}

export interface NavDest {
  label: string
  path: string
  icon: string
  keywords: string
}

export const CREATE_SURFACES: CreateSurfaceDef[] = [
  { id: 'view', label: 'View', icon: 'file', shapes: ['list', 'form', 'detail', 'dashboard', 'tabs', 'blank'], shapeLabel: 'Starting shape' },
  { id: 'skill', label: 'Skill', icon: 'sparkles', shapes: ['task', 'reference', 'workflow'], shapeLabel: 'Skill kind' },
  { id: 'agent', label: 'Agent', icon: 'users', shapes: ['specialist', 'generalist', 'reviewer'], shapeLabel: 'Agent kind' },
  { id: 'pipeline', label: 'Pipeline', icon: 'play', shapes: ['sequential', 'fanout', 'review-gated'], shapeLabel: 'Topology' },
  { id: 'trigger', label: 'Trigger', icon: 'zap', shapes: ['webhook', 'schedule', 'file-change'], shapeLabel: 'Kind' },
  { id: 'theme', label: 'Theme', icon: 'palette', shapes: ['light', 'dark', 'brand-accent', 'high-contrast'], shapeLabel: 'Starting shape' },
]

export const NAV_DESTINATIONS: NavDest[] = [
  { label: 'Dashboard', path: '/dashboard', icon: 'sparkles', keywords: 'home overview' },
  { label: 'Pipelines', path: '/pipelines', icon: 'play', keywords: 'workflows runs jobs' },
  { label: 'Agents', path: '/agents', icon: 'users', keywords: 'configure profiles' },
  { label: 'Skills', path: '/skills', icon: 'zap', keywords: 'browse edit files' },
  { label: 'Git', path: '/git', icon: 'git', keywords: 'status diff commit branch' },
  { label: 'Collections', path: '/collections', icon: 'folder', keywords: 'data items query' },
  { label: 'Canvas', path: '/canvas', icon: 'image', keywords: 'docs visual feedback' },
  { label: 'Events', path: '/events', icon: 'sparkles', keywords: 'stream log kernel' },
  { label: 'Settings', path: '/settings', icon: 'settings', keywords: 'config theme provider' },
  { label: 'Chat', path: '/chat', icon: 'message-square-plus', keywords: 'agent conversation' },
  { label: 'Specs', path: '/specs', icon: 'file', keywords: 'features design' },
  { label: 'Theme Studio', path: '/theme-studio', icon: 'palette', keywords: 'design tokens' },
  { label: 'Chats (Mission Control)', path: '/chats', icon: 'messages-square', keywords: 'sessions active' },
]

const KEBAB_RE = /^[a-z][a-z0-9-]{0,48}$/
export function isKebab(s: string): boolean { return KEBAB_RE.test(s) }

export function detectMode(value: string, fallback: BarMode): BarMode {
  if (value.startsWith('/')) return 'command'
  if (value.startsWith('>')) return 'find'
  if (value.startsWith('+')) return 'create'
  if (value.trim().length === 0) return fallback
  if (/(^|\s)@[^\s@]*$/.test(value)) return 'mention'
  return 'ai'
}

function esc(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;')
}

export interface SlashCommand {
  key: string
  label: string
  hint: string
}

export const SLASH_COMMANDS: SlashCommand[] = [
  { key: 'new', label: '/new', hint: 'Scaffold a view, skill, agent, …' },
  { key: 'chats', label: '/chats', hint: 'Open Mission Control' },
  { key: 'settings', label: '/settings', hint: 'Jump to settings' },
  { key: 'lint', label: '/lint', hint: 'Run lint on the current view' },
  { key: 'end', label: '/end', hint: 'End the current session' },
  { key: 'close', label: '/close', hint: 'Close the command bar' },
]

export function renderInputRow(state: {
  mode: BarMode
  value: string
  session: SessionSummary | null
}): string {
  const glyphs: Record<BarMode, { text: string; ai: boolean; placeholder: string }> = {
    command: { text: '/', ai: false, placeholder: 'Run a Ship action…' },
    find:    { text: '>', ai: false, placeholder: 'Jump to a view…' },
    mention: { text: '@', ai: false, placeholder: 'Reference an agent, view, file…' },
    create:  { text: '+', ai: false, placeholder: 'Create a view, skill, agent…' },
    ai:      { text: 'ai', ai: true, placeholder: 'Ask Ship or type / for actions' },
  }
  const g = glyphs[state.mode]
  const statusClass = state.session?.status === 'thinking' ? 'thinking' : state.session ? 'active' : ''
  const statusBg = statusClass === 'thinking' ? 'var(--warning,#d97706)' : statusClass === 'active' ? 'var(--success,#16a34a)' : 'var(--muted-foreground)'
  const agentLabel = esc(state.session?.agent_id ?? 'No session')

  return `
<div class="cb-input-row">
  <span class="cb-mode-glyph${g.ai ? ' ai' : ''}" aria-hidden="true">${esc(g.text)}</span>
  <input
    class="cb-search"
    type="text"
    autocomplete="off"
    spellcheck="false"
    placeholder="${esc(g.placeholder)}"
    value="${esc(state.value)}"
    aria-label="Command bar search"
    aria-autocomplete="list"
  />
  <kbd class="cb-esc" aria-label="Escape to close">esc</kbd>
  <div class="cb-chip-wrap">
    <span class="cb-session-chip" role="button" aria-haspopup="listbox" title="${agentLabel}">
      <span class="cb-dot" style="background:${statusBg}"></span>
      <span>${agentLabel}</span>
    </span>
    <div class="cb-session-popover" style="display:none" role="listbox" aria-label="Sessions"></div>
  </div>
</div>`
}

export function renderSessionPopover(
  allSessions: SessionSummary[],
  currentId: string | undefined,
): string {
  const active = allSessions.filter((s) => s.status !== 'ended' && s.agent_id)
  let rows = active.length === 0
    ? `<div class="cb-session-row"><span></span><span class="meta">No active sessions</span><span></span></div>`
    : active.map((s) => {
        const bg = s.status === 'thinking' ? 'var(--warning,#d97706)' : 'var(--success,#16a34a)'
        const cur = s.id === currentId ? ' current' : ''
        return `<div class="cb-session-row${cur}" data-session-id="${esc(s.id)}">
          <span class="cb-dot" style="background:${bg}"></span>
          <span><strong>${esc(s.agent_id ?? '')}</strong><span class="meta"> · ${esc(s.provider ?? '')} · ${esc(s.workspace_id)}</span></span>
          <button class="cb-session-end" data-end-id="${esc(s.id)}" title="End session" aria-label="End session ${esc(s.agent_id ?? '')}">×</button>
        </div>`
      }).join('')
  return `${rows}
<div class="cb-session-sep" role="separator"></div>
<div class="cb-session-action" data-action="new-session">
  <sui-icon name="plus" size="sm" aria-hidden="true"></sui-icon>
  <span>Start new session</span>
</div>
<div class="cb-session-action" data-action="view-chats">
  <sui-icon name="columns" size="sm" aria-hidden="true"></sui-icon>
  <span>View all chats</span>
</div>`
}

export function renderFindBody(value: string, selectedIdx: number): string {
  const needle = value.replace(/^>+\s*/, '').toLowerCase().trim()
  const items = NAV_DESTINATIONS.filter((d) =>
    !needle ||
    d.label.toLowerCase().includes(needle) ||
    d.keywords.toLowerCase().includes(needle) ||
    d.path.toLowerCase().includes(needle)
  )
  if (items.length === 0) return `<div class="cb-empty">No matches.</div>`
  const rows = items.map((d, i) =>
    `<div class="cb-item" role="option" aria-selected="${i === selectedIdx ? 'true' : 'false'}" data-path="${esc(d.path)}">
      <sui-icon name="${esc(d.icon)}" size="sm" aria-hidden="true"></sui-icon>
      <span>${esc(d.label)}</span>
      <span class="hint">${esc(d.path)}</span>
    </div>`
  ).join('')
  return `<div class="cb-mode-hint" aria-label="Jump to">Jump to</div><div class="cb-list" role="listbox">${rows}</div>`
}

export function renderCommandBody(value: string, selectedIdx: number): string {
  const needle = value.replace(/^\/+\s*/, '').toLowerCase().trim()
  const items = SLASH_COMMANDS.filter((c) =>
    !needle ||
    c.key.includes(needle) ||
    c.label.toLowerCase().includes(needle) ||
    c.hint.toLowerCase().includes(needle)
  )
  if (items.length === 0) return `<div class="cb-empty">No commands match.</div>`
  const rows = items.map((c, i) =>
    `<div class="cb-item" role="option" aria-selected="${i === selectedIdx ? 'true' : 'false'}" data-cmd="${esc(c.key)}">
      <span style="font-family:var(--font-mono)">${esc(c.label)}</span>
      <span></span>
      <span class="hint">${esc(c.hint)}</span>
    </div>`
  ).join('')
  return `<div class="cb-mode-hint">Actions</div><div class="cb-list" role="listbox">${rows}</div>`
}

export function renderMentionBody(
  value: string,
  views: string[],
  agents: AgentSummary[],
  sessions: SessionSummary[],
  selectedIdx: number,
): string {
  const needle = value.replace(/^@+/, '').toLowerCase().trim()
  type Ref = { kind: 'view' | 'agent' | 'session'; id: string; label: string }
  const refs: Ref[] = [
    ...views.map((v) => ({ kind: 'view' as const, id: v, label: v })),
    ...agents.map((a) => ({ kind: 'agent' as const, id: a.id, label: a.id })),
    ...sessions
      .filter((s) => s.status !== 'ended' && s.agent_id)
      .map((s) => ({ kind: 'session' as const, id: s.id, label: `${s.agent_id} (session)` })),
  ]
  const items = refs.filter((r) => !needle || r.id.toLowerCase().includes(needle)).slice(0, 40)
  if (items.length === 0) return `<div class="cb-empty">No matching views or agents.</div>`
  const iconName = (kind: string) => kind === 'view' ? 'app-window' : kind === 'agent' ? 'bot' : 'message-square-plus'
  const rows = items.map((r, i) =>
    `<div class="cb-item" role="option" aria-selected="${i === selectedIdx ? 'true' : 'false'}" data-mention-kind="${esc(r.kind)}" data-mention-id="${esc(r.id)}">
      <sui-icon name="${esc(iconName(r.kind))}" size="sm" aria-hidden="true"></sui-icon>
      <span>${esc(r.label)}</span>
      <span class="hint">${esc(r.kind)}</span>
    </div>`
  ).join('')
  return `<div class="cb-mode-hint">Reference</div><div class="cb-list" role="listbox">${rows}</div>`
}

export function renderCreateBody(state: {
  surface: CreateSurface
  name: string
  purpose: string
  shape: string
  submitting: boolean
  error: string | null
}): string {
  const def = CREATE_SURFACES.find((s) => s.id === state.surface)!
  const surfaceBtns = CREATE_SURFACES.map((s) =>
    `<button class="cb-surface-btn" type="button" aria-pressed="${s.id === state.surface ? 'true' : 'false'}" data-surface="${esc(s.id)}">
      <sui-icon name="${esc(s.icon)}" aria-hidden="true"></sui-icon>
      <span>${esc(s.label)}</span>
    </button>`
  ).join('')
  const shapeOptions = def.shapes.map((sh) =>
    `<option value="${esc(sh)}"${sh === state.shape ? ' selected' : ''}>${esc(sh)}</option>`
  ).join('')
  const nameValid = KEBAB_RE.test(state.name)
  const errorHtml = state.error ? `<div class="cb-create-error">${esc(state.error)}</div>` : ''
  return `
<div class="cb-mode-hint">Create</div>
<div class="cb-create-surfaces" role="group" aria-label="Surface type">${surfaceBtns}</div>
<div class="cb-create-grid">
  <label>
    <span>Name (kebab-case)</span>
    <input type="text" class="cb-create-name" placeholder="my-${esc(def.id)}" value="${esc(state.name)}" autocomplete="off" />
  </label>
  <label>
    <span>${esc(def.shapeLabel)}</span>
    <select class="cb-create-shape">${shapeOptions}</select>
  </label>
  <label class="span-2">
    <span>Purpose (optional)</span>
    <input type="text" class="cb-create-purpose" placeholder="One-line description" value="${esc(state.purpose)}" autocomplete="off" />
  </label>
</div>
${errorHtml}
<div class="cb-create-actions">
  <button class="cb-create-submit" type="button" ${state.submitting || !nameValid ? 'disabled' : ''}>
    ${state.submitting ? 'Creating…' : `Create ${esc(def.label)}`}
  </button>
</div>`
}

export function renderSpawnCard(state: {
  agents: AgentSummary[]
  providers: string[]
  spawnAgent: string
  spawnProvider: string
  spawning: boolean
  spawnError: string | null
}): string {
  const agentOpts = state.agents.length === 0
    ? `<option value="">No agents found</option>`
    : state.agents.map((a) =>
        `<option value="${esc(a.id)}"${a.id === state.spawnAgent ? ' selected' : ''}>${esc(a.name ?? a.label ?? a.id)}</option>`
      ).join('')
  const provOpts = state.providers.length === 0
    ? `<option value="">No providers installed</option>`
    : state.providers.map((p) =>
        `<option value="${esc(p)}"${p === state.spawnProvider ? ' selected' : ''}>${esc(p)}</option>`
      ).join('')
  const errHtml = state.spawnError ? `<div class="cb-spawn-error">${esc(state.spawnError)}</div>` : ''
  const disabled = state.spawning || !state.spawnAgent || !state.spawnProvider
  return `
<div class="cb-spawn-card">
  <div class="cb-spawn-title">Start a session</div>
  <div class="cb-spawn-row">
    <label>Agent</label>
    <select class="cb-spawn-agent">${agentOpts}</select>
  </div>
  <div class="cb-spawn-row">
    <label>Provider</label>
    <select class="cb-spawn-provider">${provOpts}</select>
  </div>
  ${errHtml}
  <button class="cb-spawn-start" type="button" ${disabled ? 'disabled' : ''}>
    ${state.spawning ? 'Starting…' : 'Start session'}
  </button>
</div>`
}

export function renderThread(thread: ThreadLine[]): string {
  if (thread.length === 0) {
    return `<div class="cb-thread"><div class="cb-thread-empty">Type above and press Enter. The agent's replies land here.</div></div>`
  }
  const turns = thread.map((m) =>
    `<div class="cb-turn" data-sender="${esc(m.sender)}">
      <span class="sender">${esc(m.sender)}</span>
      <span class="text">${esc(m.text)}</span>
    </div>`
  ).join('')
  return `<div class="cb-thread">${turns}</div>`
}

export function renderChatLayout(state: {
  session: SessionSummary
  thread: ThreadLine[]
  value: string
  sessionPickerOpen: boolean
  allSessions: SessionSummary[]
}): string {
  const statusClass = state.session.status === 'thinking' ? 'thinking' : 'active'
  const bg = statusClass === 'thinking' ? 'var(--warning,#d97706)' : 'var(--success,#16a34a)'
  const messagesHtml = state.thread.length === 0
    ? `<div class="cb-chat-empty">No messages yet. Type below to send.</div>`
    : state.thread.map((m) =>
        `<div class="cb-chat-turn" data-sender="${esc(m.sender)}"><div class="cb-chat-bubble">${esc(m.text)}</div></div>`
      ).join('')
  const popoverHtml = state.sessionPickerOpen
    ? `<div class="cb-session-popover" style="display:block" role="listbox">${renderSessionPopover(state.allSessions, state.session.id)}</div>`
    : `<div class="cb-session-popover" style="display:none" role="listbox"></div>`
  return `
<div class="cb-chat-header">
  <span class="cb-dot" style="background:${bg}" aria-hidden="true"></span>
  <div style="display:grid;line-height:1.25">
    <span class="title">${esc(state.session.agent_id ?? '')}</span>
    <span class="sub">${esc(state.session.provider ?? '—')} · ${esc(state.session.workspace_id)}</span>
  </div>
  <span style="flex:1"></span>
  <div class="cb-chip-wrap">
    <span class="cb-session-chip" role="button" aria-haspopup="listbox" style="border-left:0">Switch</span>
    ${popoverHtml}
  </div>
  <kbd class="cb-esc" aria-label="Escape to close">esc</kbd>
</div>
<div class="cb-chat-messages">${messagesHtml}</div>
<div class="cb-chat-composer">
  <textarea class="cb-chat-input" rows="2" placeholder="Message the agent…">${esc(state.value)}</textarea>
  <button class="cb-chat-send" type="button" ${!state.value.trim() ? 'disabled' : ''}>Send</button>
</div>`
}
