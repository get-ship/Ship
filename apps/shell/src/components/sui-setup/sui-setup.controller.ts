import { controller, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-setup.sui'
import { ops } from '../../ops'

const STORAGE_KEY = 'ship-setup-state'
const TOTAL_STEPS = 6
const PROVIDERS = [
  { id: 'claude',   name: 'Claude',   sub: 'Anthropic · claude' },
  { id: 'codex',    name: 'Codex',    sub: 'OpenAI · codex' },
  { id: 'gemini',   name: 'Gemini',   sub: 'Google · gemini' },
  { id: 'opencode', name: 'OpenCode', sub: 'Open source · opencode' },
  { id: 'cursor',   name: 'Cursor',   sub: 'Cursor · cursor-agent' },
] as const

interface PersistedState {
  step: number
  identityName: string
  provider: string
  providerValidated: boolean
  projectName: string
  projectPath: string
  sampleApp: boolean
  completed: boolean
}

function defaultProjectPath(name: string): string {
  const trimmed = name.trim() || '<name>'
  return `~/Documents/ShipProjects/${trimmed}`
}

function loadPersisted(): Partial<PersistedState> {
  try {
    const raw = localStorage.getItem(STORAGE_KEY)
    if (!raw) return {}
    const parsed = JSON.parse(raw)
    return parsed && typeof parsed === 'object' ? parsed : {}
  } catch {
    return {}
  }
}

function savePersisted(state: PersistedState): void {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
  } catch { /* best-effort */ }
}

function readEntry(s: Record<string, unknown>): PersistedState {
  // `s` is the MutableState proxy: indexing a state key returns the UNWRAPPED
  // value (the proxy already calls the signal). Read it directly — do NOT call
  // it (the old `(s[k])()` invoked a number/string and crashed at runtime).
  const sig = <T>(k: string): T => s[k] as T
  return {
    step: sig<number>('step'),
    identityName: sig<string>('identityName'),
    provider: sig<string>('provider'),
    providerValidated: sig<boolean>('providerTestOk'),
    projectName: sig<string>('projectName'),
    projectPath: sig<string>('projectPath'),
    sampleApp: sig<boolean>('sampleApp'),
    completed: false,
  }
}

async function fetchInstalledProviders(): Promise<string[]> {
  try {
    const res = await fetch('/api/v1/providers/installed', { credentials: 'same-origin' })
    if (!res.ok) return []
    const json = await res.json()
    const data = Array.isArray(json?.data) ? json.data : json
    return Array.isArray(data?.providers) ? data.providers : []
  } catch {
    return []
  }
}

async function postProject(name: string, path: string): Promise<{ ok: boolean; error?: string; warning?: string }> {
  try {
    await ops.project.add({ path, name })
    return { ok: true }
  } catch (e) {
    const msg = (e as Error).message ?? 'add failed'
    if (/not a directory|exists/i.test(msg)) {
      return { ok: false, warning: msg }
    }
    return { ok: false, error: msg }
  }
}

function persistFromState(s: Record<string, unknown>, completed = false): void {
  const entry = readEntry(s)
  entry.completed = completed
  savePersisted(entry)
}

// Listener teardown ref (set in mount, called in unmount). Module-local
// because the chrome scope proxy rejects extra property writes.
let setupKeyCleanup: (() => void) | null = null

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const persisted = loadPersisted()
    return {
      step: signal<number>(persisted.completed ? 1 : (persisted.step ?? 1)),
      open: signal<boolean>(true),
      identityName: signal<string>(persisted.identityName ?? ''),
      identityError: signal<string>(''),
      provider: signal<string>(persisted.provider ?? ''),
      providerTestOk: signal<boolean>(persisted.providerValidated ?? false),
      providerTesting: signal<boolean>(false),
      providerError: signal<string>(''),
      providerOkLabel: signal<string>(persisted.providerValidated ? 'Ready' : ''),
      projectName: signal<string>(persisted.projectName ?? ''),
      projectPath: signal<string>(persisted.projectPath ?? ''),
      projectError: signal<string>(''),
      projectWarning: signal<string>(''),
      projectSubmitting: signal<boolean>(false),
      sampleApp: signal<boolean>(persisted.sampleApp ?? true),
      confirmingExit: signal<boolean>(false),
      providerList: PROVIDERS,
    }
  },
  on: {
    goNext: (s) => {
      const next = Math.min(TOTAL_STEPS, s.step + 1)
      s.step = next
      persistFromState(s as unknown as Record<string, unknown>)
    },
    goBack: (s) => {
      const prev = Math.max(1, s.step - 1)
      s.step = prev
      persistFromState(s as unknown as Record<string, unknown>)
    },
    submitIdentity: (s) => {
      const name = String(s.identityName ?? '').trim()
      if (!name) {
        s.identityError = 'Please enter a display name.'
        return
      }
      s.identityError = ''
      s.step = 3
      persistFromState(s as unknown as Record<string, unknown>)
    },
    selectProvider: (s, id: string) => {
      s.provider = id
      s.providerTestOk = false
      s.providerError = ''
      s.providerOkLabel = ''
      // Fire validation asynchronously.
      void (async () => {
        s.providerTesting = true
        const installed = await fetchInstalledProviders()
        s.providerTesting = false
        if (installed.includes(id)) {
          s.providerTestOk = true
          s.providerOkLabel = `${id} CLI detected`
          s.providerError = ''
        } else {
          s.providerTestOk = false
          s.providerError = `${id} CLI not found on PATH. Install it, then retry — or pick another provider.`
        }
        persistFromState(s as unknown as Record<string, unknown>)
      })()
    },
    validateProvider: (s) => {
      const id = String(s.provider ?? '')
      if (!id) return
      // Re-trigger by clearing then re-selecting through the same path.
      s.providerTestOk = false
      s.providerError = ''
      void (async () => {
        s.providerTesting = true
        const installed = await fetchInstalledProviders()
        s.providerTesting = false
        if (installed.includes(id)) {
          s.providerTestOk = true
          s.providerOkLabel = `${id} CLI detected`
        } else {
          s.providerError = `${id} CLI not found on PATH. Install it, then retry — or pick another provider.`
        }
        persistFromState(s as unknown as Record<string, unknown>)
      })()
    },
    submitProvider: (s) => {
      // Provider may be unvalidated (CLI missing) but user can still continue;
      // the wizard captures the choice and Settings can finish credentials.
      s.step = 4
      persistFromState(s as unknown as Record<string, unknown>)
    },
    submitProject: (s) => {
      const name = String(s.projectName ?? '').trim()
      if (!name) return
      const path = String(s.projectPath ?? '').trim() || defaultProjectPath(name)
      s.projectPath = path
      s.projectError = ''
      s.projectWarning = ''
      s.projectSubmitting = true
      void (async () => {
        const result = await postProject(name, path)
        s.projectSubmitting = false
        if (result.ok) {
          s.step = 5
          persistFromState(s as unknown as Record<string, unknown>)
          return
        }
        if (result.warning) {
          s.projectWarning = result.warning
        } else if (result.error) {
          s.projectError = result.error
        }
        // Advance anyway — capture-now, fix-in-settings.
        s.step = 5
        persistFromState(s as unknown as Record<string, unknown>)
      })()
    },
    submitSample: (s) => {
      s.step = 6
      persistFromState(s as unknown as Record<string, unknown>)
    },
    finish: (s) => {
      persistFromState(s as unknown as Record<string, unknown>, true)
      try { localStorage.setItem('ship-setup-completed', '1') } catch { /* best-effort */ }
      window.location.href = '/'
    },
    requestExit: (s) => { s.confirmingExit = true },
    cancelExit: (s) => { s.confirmingExit = false },
    confirmExit: (s) => {
      s.confirmingExit = false
      persistFromState(s as unknown as Record<string, unknown>, true)
      try { localStorage.setItem('ship-setup-completed', '1') } catch { /* best-effort */ }
      window.location.href = '/'
    },
    onFieldKeydown: (s, e: KeyboardEvent) => {
      if (e.key === 'Escape') {
        const target = e.target as (HTMLInputElement & { blur(): void }) | null
        target?.blur()
        e.stopPropagation()
        return
      }
      if (e.key === 'Enter') {
        e.preventDefault()
        const step = s.step
        if (step === 2) (s as unknown as Record<string, () => void>).submitIdentity?.()
        else if (step === 4) (s as unknown as Record<string, () => void>).submitProject?.()
      }
    },
  },
  mount: (state, _el) => {
    // Cmd+. (or Ctrl+.) dismisses the wizard with confirmation.
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === '.') {
        e.preventDefault()
        state.confirmingExit = true
        return
      }
      // Top-level Enter advances the welcome / sample / done steps.
      if (e.key === 'Enter' && !(e.target instanceof HTMLInputElement)) {
        const step = state.step
        if (step === 1) (state as unknown as Record<string, () => void>).goNext?.()
        else if (step === 5) (state as unknown as Record<string, () => void>).submitSample?.()
        else if (step === 6) (state as unknown as Record<string, () => void>).finish?.()
      }
    }
    window.addEventListener('keydown', onKey, true)
    // Chrome scope is a guarded proxy — can't stash _cleanup on it the way the
    // daemon view host allowed. Keep the listener teardown module-local.
    setupKeyCleanup = () => window.removeEventListener('keydown', onKey, true)
  },
  unmount: () => {
    setupKeyCleanup?.()
    setupKeyCleanup = null
  },
})
