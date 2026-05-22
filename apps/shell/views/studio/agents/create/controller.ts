import { controller, signal, derived, live } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

type TemplateEntry = {
  id: string
  label: string
  description: string
  letter: string
  hue: string
  skill_ids: string[]
  permission_preset: string
}

type CreateCtxValue = {
  templates: TemplateEntry[]
  permission_presets: Array<{ id: string; label: string; description: string }>
  available_providers: string[]
}

function slugify(input: string): string {
  return input
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/(^-|-$)/g, '')
}

function isValidId(id: string): boolean {
  return /^[a-z0-9]+(?:-[a-z0-9]+)*$/.test(id)
}

export default controller<Scope>({
  scope: ({ ship }) => {
    const createCtx = live(() => ship.agents.create_ctx())
    const step = signal(0)
    const template = signal('blank')
    const templateSkills = signal<string[]>([])
    const hue = signal('orange')
    const name = signal('')
    const id = signal('')
    const desc = signal('')
    const providers = signal<string[]>(['claude'])
    const preset = signal('standard')
    const creating = signal(false)
    const err = signal('')

    // Resolved spec preview shown on the review step. Mirrors the doc
    // shape the daemon will write to disk; helps users sanity-check
    // before they commit.
    const resolvedDoc = derived(() => {
      const doc: Record<string, unknown> = {
        $schema: 'https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json',
        schema_version: '1',
        agent: {
          id: id(),
          name: name() || id(),
          description: desc(),
          providers: providers(),
          tags: [],
        },
        permissions: { preset: preset() },
      }
      const skills = templateSkills()
      if (skills.length > 0) doc.skills = { refs: skills }
      return doc
    })

    const reviewJson = derived(() => JSON.stringify(resolvedDoc(), null, 2))

    const idValid = derived(() => isValidId(id()))
    const idError = derived(() => {
      const v = id()
      if (!v) return 'ID is required.'
      if (!idValid()) return 'Use lowercase letters, digits, and hyphens (no leading/trailing dash).'
      return ''
    })
    const stepValid = derived(() => {
      switch (step()) {
        case 0: return !!template()
        case 1: return idValid() && !!(name() || id())
        case 2: return providers().length > 0 && !!preset()
        case 3: return idValid()
        default: return false
      }
    })

    return {
      createCtx, step, template, templateSkills, hue, name, id, desc, providers, preset,
      creating, err,
      resolvedDoc, reviewJson, idValid, idError, stepValid,
    }
  },
  on: {
    cancel: ({ navigate }) => navigate('/studio/agents'),
    back: (s) => { s.step = Math.max(0, (s.step as number) - 1) },
    next: (s) => {
      if (!s.stepValid) return
      s.step = Math.min(3, (s.step as number) + 1)
    },
    goTo: (s, n: number) => { if (n <= (s.step as number)) s.step = n },
    selectTemplate: (s, t: TemplateEntry) => {
      s.template = t.id
      s.preset = t.permission_preset
      s.hue = t.hue
      s.templateSkills = Array.isArray(t.skill_ids) ? t.skill_ids : []
    },
    toggleProvider: (s, p: string) => {
      const cur = s.providers as string[]
      s.providers = cur.includes(p) ? cur.filter(x => x !== p) : [...cur, p]
    },
    nameInput: (s, e: CustomEvent<{ value: string }>) => {
      s.name = e.detail.value
      if (!(s.id as string) && e.detail.value) {
        s.id = slugify(e.detail.value)
      }
    },
    idInput: (s, e: CustomEvent<{ value: string }>) => {
      s.id = slugify(e.detail.value)
    },
    descInput: (s, e: CustomEvent<{ value: string }>) => { s.desc = e.detail.value },
    create: async (s) => {
      if (!s.stepValid) return
      s.creating = true
      s.err = ''
      try {
        const resp = await fetch('/api/v1/agents', {
          method: 'POST',
          credentials: 'include',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ id: s.id, doc: s.resolvedDoc }),
        })
        if (resp.ok) {
          const body = (await resp.json()) as { id: string }
          window.location.href = '/studio/agents/' + body.id
          return
        }
        const body = (await resp.json().catch(() => ({}))) as { message?: string; error?: string }
        s.err = body.message || body.error || `Create failed (${resp.status})`
      } catch (e) {
        s.err = (e as Error).message ?? 'Network error'
      } finally {
        s.creating = false
      }
    },
  },
})

// Used by the Scope type generator — re-exported so the .sui template
// import doesn't lose track of the create-ctx shape during HMR.
export type { CreateCtxValue }
