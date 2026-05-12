import { controller, signal, unwrap } from '@ship/sui-runtime'
import type { Scope } from './page.sui'

export default controller<Scope>({
  scope: () => {
    const step = signal(0)
    const template = signal('blank')
    const hue = signal('orange')
    const name = signal('')
    const id = signal('')
    const desc = signal('')
    const providers = signal<string[]>(['claude'])
    const preset = signal('standard')
    const creating = signal(false)
    const err = signal('')
    return { step, template, hue, name, id, desc, providers, preset, creating, err }
  },
  on: {
    cancel: ({ navigate }) => navigate('/agents'),
    back: (s) => { s.step = Math.max(0, (s.step as number) - 1) },
    next: (s) => { s.step = Math.min(3, (s.step as number) + 1) },
    goTo: (s, n: number) => { if (n <= (s.step as number)) s.step = n },
    selectTemplate: (s, tId: string, tPreset: string, tHue: string) => {
      s.template = tId
      s.preset = tPreset
      s.hue = tHue
    },
    toggleProvider: (s, p: string) => {
      const cur = s.providers as string[]
      s.providers = cur.includes(p) ? cur.filter(x => x !== p) : [...cur, p]
    },
    nameInput: (s, e: CustomEvent<{ value: string }>) => {
      s.name = e.detail.value
      if (!(s.id as string) && e.detail.value) {
        s.id = e.detail.value.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '')
      }
    },
    create: (s) => {
      s.creating = true
      s.err = ''
      fetch('/api/v1/agents', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        credentials: 'include',
        body: JSON.stringify({
          id: s.id,
          doc: {
            '$schema': 'https://raw.githubusercontent.com/madvib/ship/main/schemas/agent.schema.json',
            schema_version: '1',
            agent: { id: s.id, name: s.name, description: s.desc, providers: s.providers },
            permissions: { preset: s.preset },
          },
        }),
      })
        .then(res => res.ok
          ? res.json().then((b: { id: string }) => { window.location.href = '/agents/' + b.id })
          : res.json().then((b: { message?: string }) => { s.creating = false; s.err = b.message || 'Create failed' }))
        .catch(() => { s.creating = false; s.err = 'Network error' })
    },
  },
})
