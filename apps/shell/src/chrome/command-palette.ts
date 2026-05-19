// Command palette wiring — the shell side of <sui-command-palette>.
//
// The palette primitive (@ship/primitives) is generic: it renders the
// command groups it is handed and reports the user's pick. This module
// supplies those groups and acts on the selection. It is the receiver for
// the `sui-open-command-palette` intent the shell chrome emits on ⌘K.
//
// Three groups:
//   Open     — static navigation rows (Studio · Sessions · Events · Git).
//   Run      — dynamic; agents + skills pulled from the SDK on first open.
//   Settings — static (Switch theme… · Open Settings).
//
// Selection routing keys off the item's `value`:
//   - a path ("/studio")     → navigate
//   - "run:agent:<id>"       → dispatch `sui:run` { kind:'agent', id }
//   - "run:skill:<id>"       → dispatch `sui:run` { kind:'skill', id }
//   - "action:theme"         → dispatch `sui:theme-picker:open`

import { signal, effect, type LiveSource, type LiveState } from '@ship/sui-runtime'

/** One selectable row — mirrors the primitive's `CommandItem`. */
interface CommandItem {
  id: string
  label: string
  icon?: string
  hotkey?: string
  meta?: string
  value?: string
}
interface CommandGroup {
  label: string
  items: CommandItem[]
}

/** The palette element's scriptable surface (set by the primitive). */
interface PaletteEl extends HTMLElement {
  commands: CommandGroup[]
  show?: () => void
}

interface AgentRow { id: string; name?: string }
interface SkillRow { id: string; name?: string; description?: string }

/** Minimal slice of the SDK this module needs — keeps the dependency
 *  explicit and the module testable without the full client. */
export interface PaletteSdk {
  agents: { summaries(): LiveSource<{ items?: AgentRow[] }> }
  skills: { list(): LiveSource<{ items?: SkillRow[] }> }
}

export interface CommandPaletteDeps {
  sdk: PaletteSdk
  navigate: (path: string) => void
}

const PALETTE_ID = 'ship-command-palette'

/** The static groups — present the moment the palette opens. */
function staticGroups(): CommandGroup[] {
  return [
    {
      label: 'Open',
      items: [
        { id: 'studio', label: 'Open Studio', icon: 'grid', hotkey: '⌘.', meta: 'configure agents, skills, themes…', value: '/studio' },
        { id: 'sessions', label: 'Open Sessions', icon: 'activity', meta: 'live runs', value: '/sessions' },
        { id: 'events', label: 'Open Events', icon: 'radio', meta: 'live stream', value: '/events' },
        { id: 'git', label: 'Open Git', icon: 'git-branch', meta: 'branch + diff', value: '/git' },
      ],
    },
    {
      label: 'Settings',
      items: [
        { id: 'theme', label: 'Switch theme…', icon: 'palette', hotkey: '⌘T', value: 'action:theme' },
        { id: 'settings', label: 'Open Settings', icon: 'settings', hotkey: '⌘,', value: '/settings' },
      ],
    },
  ]
}

/** Resolve the first non-loading frame from a LiveSource, then close the
 *  channel. Errors and the timeout resolve to an empty result rather than
 *  rejecting — a missing dynamic group must not break the palette. */
function firstValue<T>(source: LiveSource<T>, timeoutMs = 4000): Promise<T | null> {
  return new Promise((resolve) => {
    const sig = signal<LiveState<T>>({ kind: 'loading' })
    let dispose = () => {}
    let done = false
    const finish = (value: T | null) => {
      if (done) return
      done = true
      clearTimeout(timer)
      stop()
      dispose()
      resolve(value)
    }
    const stop = effect(() => {
      const s = sig()
      if (s.kind === 'ok') finish(s.value)
      else if (s.kind === 'error') finish(null)
    })
    const timer = setTimeout(() => finish(null), timeoutMs)
    try {
      dispose = source.subscribe(sig)
    } catch {
      finish(null)
    }
  })
}

/** Build the dynamic "Run" group from the SDK. Stable alphabetical order
 *  within each kind (agents then skills); pipelines have no list endpoint
 *  yet, so they're omitted until one lands. */
async function loadRunGroup(sdk: PaletteSdk): Promise<CommandGroup> {
  const [agents, skills] = await Promise.all([
    firstValue(sdk.agents.summaries()),
    firstValue(sdk.skills.list()),
  ])
  const items: CommandItem[] = []
  for (const a of (agents?.items ?? []).slice().sort((x, y) => x.id.localeCompare(y.id))) {
    items.push({ id: `agent:${a.id}`, label: `Run agent ${a.id}`, icon: 'bot', hotkey: '⏎', value: `run:agent:${a.id}` })
  }
  for (const s of (skills?.items ?? []).slice().sort((x, y) => x.id.localeCompare(y.id))) {
    items.push({ id: `skill:${s.id}`, label: `Invoke skill ${s.id}`, icon: 'zap', hotkey: '⏎', value: `run:skill:${s.id}` })
  }
  return { label: 'Run', items }
}

/** Map a selected item's value onto a shell action. */
function routeSelection(value: string | undefined, navigate: (p: string) => void): void {
  if (!value) return
  if (value.startsWith('/')) {
    navigate(value)
    return
  }
  const runMatch = /^run:(agent|skill):(.+)$/.exec(value)
  if (runMatch) {
    document.dispatchEvent(
      new CustomEvent('sui:run', { detail: { kind: runMatch[1], id: runMatch[2] }, bubbles: true }),
    )
    return
  }
  if (value === 'action:theme') {
    document.dispatchEvent(new CustomEvent('sui:theme-picker:open', { bubbles: true }))
  }
}

/** Ensure a single palette element exists on `document.body`. */
export function ensurePaletteElement(): PaletteEl {
  let el = document.getElementById(PALETTE_ID) as PaletteEl | null
  if (!el) {
    el = document.createElement('sui-command-palette') as PaletteEl
    el.id = PALETTE_ID
    document.body.appendChild(el)
  }
  return el
}

/**
 * Mount the palette and wire it to the shell. Idempotent enough for HMR:
 * the element is reused if already present. Returns a disposer that
 * removes the document-level listeners (the element stays mounted).
 */
export function initCommandPalette(deps: CommandPaletteDeps): () => void {
  const el = ensurePaletteElement()
  el.commands = staticGroups()

  let runLoaded = false
  let runLoading = false

  // Lazy-load the dynamic Run group the first time the palette opens.
  async function loadRun(): Promise<void> {
    if (runLoaded || runLoading) return
    runLoading = true
    el.setAttribute('loading', '')
    try {
      const run = await loadRunGroup(deps.sdk)
      const groups = staticGroups()
      // Insert Run between Open and Settings to match the design order.
      groups.splice(1, 0, run)
      el.commands = groups
      runLoaded = true
    } finally {
      runLoading = false
      el.removeAttribute('loading')
    }
  }

  const onOpen = () => {
    el.setAttribute('open', '')
    void loadRun()
  }
  const onSelect = (e: Event) => {
    const detail = (e as CustomEvent<{ id: string; value?: string }>).detail
    routeSelection(detail?.value, deps.navigate)
  }

  document.addEventListener('sui-open-command-palette', onOpen)
  el.addEventListener('sui-command-select', onSelect)

  return () => {
    document.removeEventListener('sui-open-command-palette', onOpen)
    el.removeEventListener('sui-command-select', onSelect)
  }
}
