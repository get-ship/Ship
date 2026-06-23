import { controller, signal, derived, live } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './page.sui'
import { navigate } from '@ship/deck/router'
import { routes } from '../../../../routes.gen'

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

export default controller({
  scope: (ctx: ViewScope<Scope>) => {
    const ship = ctx.clients.ship
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

    // Resolved spec preview shown on the review step.
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
    cancel: () => navigate(routes.studioAgents()),
    back: (s) => { s.step = Math.max(0, s.step - 1) },
    next: (s) => {
      if (!s.stepValid) return
      s.step = Math.min(3, s.step + 1)
    },
    goTo: (s, n: number) => { if (n <= s.step) s.step = n },
    selectTemplate: (s, t: TemplateEntry) => {
      s.template = t.id
      s.preset = t.permission_preset
      s.hue = t.hue
      s.templateSkills = Array.isArray(t.skill_ids) ? t.skill_ids : []
    },
    toggleProvider: (s, p: string) => {
      const cur = s.providers
      s.providers = cur.includes(p) ? cur.filter((x: string) => x !== p) : [...cur, p]
    },
    nameInput: (s, e: CustomEvent<{ value: string }>) => {
      s.name = e.detail.value
      if (!s.id && e.detail.value) {
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
        const m = await s.clients.ops.agents.create({
          id: s.id,
          docJson: JSON.stringify(s.resolvedDoc),
          scope: '',
        })
        navigate(routes.studioAgentsId({ id: m.id }))
        return
      } catch (e) {
        s.err = (e as Error).message ?? 'Network error'
      } finally {
        s.creating = false
      }
    },
  },
})

// Re-exported for the Scope type generator.
export type { CreateCtxValue }
