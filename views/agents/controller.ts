import { controller, signal, unwrap, live, derived } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type AgentStatus = 'running' | 'completed' | 'errored'

interface EventRow { event_type: string; actor: string; timestamp: string }
interface EventsEnvelope { groups: Array<{ rows: EventRow[] }> }

const PROVIDER_MODELS: Record<string, string[]> = {
  claude: ['claude-opus-4-7', 'claude-sonnet-4-6', 'claude-haiku-4-5-20251001'],
  codex: ['o4-mini', 'o4'],
  cursor: ['gpt-5', 'gpt-5-mini'],
  gemini: ['gemini-2.5-pro', 'gemini-2.5-flash'],
  opencode: ['opencode-default'],
}

const slugify = (s: string): string =>
  s.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '')

export default controller<Scope>({
  scope: ({ ship }) => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const agentEvents = live(() => (ship as any).events.list({ event_type: 'agent.', limit: 50 }))
    // SDK mutators captured from the controller ctx so `on` handlers can
    // reach the daemon through the typed surface instead of raw fetch.
    const createAgent = (args: { id: string; doc: Record<string, unknown> }) =>
      ship.agents.create(args)
    const startSession = (args: {
      workspace_id: string
      kind: 'agent' | 'shell'
      agent_id?: string
      provider?: string
    }) => ship.sessions.start(args)
    const agentStatus = derived<Record<string, AgentStatus>>(() => {
      const env = agentEvents()
      if (env.kind !== 'ok') return {}
      const v = env.value as EventsEnvelope
      const rows: EventRow[] = (v?.groups ?? []).flatMap((g) => g.rows ?? [])
      rows.sort((a, b) => a.timestamp.localeCompare(b.timestamp))
      const map: Record<string, AgentStatus> = {}
      for (const r of rows) {
        if (!r.actor) continue
        if (r.event_type === 'agent.started') map[r.actor] = 'running'
        else if (r.event_type === 'agent.completed') map[r.actor] = 'completed'
        else if (
          r.event_type === 'agent.error' ||
          r.event_type === 'agent.errored' ||
          r.event_type === 'agent.failed'
        ) map[r.actor] = 'errored'
      }
      return map
    })

    const dialogOpen = signal(false)
    const formName = signal('')
    const formId = signal('')
    const formIdTouched = signal(false)
    const formProvider = signal('claude')
    const formModel = signal('')
    const submitting = signal(false)
    const formErr = signal('')

    const modelOptions = derived(() => PROVIDER_MODELS[formProvider()] ?? [])
    const canSubmit = derived(() =>
      !submitting() &&
      formName().trim().length > 0 &&
      /^[a-z][a-z0-9-]*$/.test(formId()) &&
      formProvider().length > 0,
    )

    return {
      agentEvents, agentStatus,
      dialogOpen, formName, formId, formIdTouched, formProvider, formModel,
      submitting, formErr, modelOptions, canSubmit,
      createAgent, startSession,
    }
  },
  derived: {
    items: ({ agents }) => unwrap(agents)?.items ?? [],
    total: ({ agents }) => unwrap(agents)?.items.length ?? 0,
  },
  on: {
    openNew: (s) => {
      s.dialogOpen = true
      s.formName = ''
      s.formId = ''
      s.formIdTouched = false
      s.formProvider = 'claude'
      s.formModel = ''
      s.formErr = ''
    },
    closeNew: (s) => { s.dialogOpen = false },
    nameInput: (s, e: CustomEvent<{ value: string }>) => {
      const v = e.detail.value
      s.formName = v
      if (!s.formIdTouched) s.formId = slugify(v)
    },
    idInput: (s, e: CustomEvent<{ value: string }>) => {
      s.formId = e.detail.value
      s.formIdTouched = true
    },
    providerChange: (s, e: CustomEvent<{ value: string }>) => {
      s.formProvider = e.detail.value
      s.formModel = ''
    },
    modelChange: (s, e: CustomEvent<{ value: string }>) => {
      s.formModel = e.detail.value
    },
    submitNew: async (s) => {
      if (!s.canSubmit) return
      s.submitting = true
      s.formErr = ''
      const id = String(s.formId)
      const name = String(s.formName).trim()
      const provider = String(s.formProvider)
      const model = String(s.formModel).trim()
      const doc: Record<string, unknown> = {
        $schema: '../../schemas/agent.schema.json',
        schema_version: '1',
        agent: { id, name, providers: [provider], ...(model ? { model } : {}) },
        permissions: { preset: 'standard' },
      }
      try {
        await (s.createAgent as (a: { id: string; doc: Record<string, unknown> }) => Promise<unknown>)({ id, doc })
        s.dialogOpen = false
        window.location.reload()
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err)
        // SDK error messages already carry the daemon's JSON envelope —
        // surface as-is so users see "agent exists" / validation text.
        s.formErr = msg.startsWith('[ship-sdk-client]') ? msg : 'Network: ' + msg
        s.submitting = false
      }
    },
    openSession: async (s, agentId: string, provider: string, evt?: Event) => {
      if (evt) { evt.preventDefault(); evt.stopPropagation() }
      try {
        await (s.startSession as (a: {
          workspace_id: string
          kind: 'agent' | 'shell'
          agent_id?: string
          provider?: string
        }) => Promise<unknown>)({
          workspace_id: 'main', kind: 'agent', agent_id: agentId, provider,
        })
        window.location.href = '/sessions'
      } catch { /* ignore */ }
    },
  },
})
