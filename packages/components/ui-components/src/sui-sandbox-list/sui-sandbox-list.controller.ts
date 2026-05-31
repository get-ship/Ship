import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-sandbox-list.sui'

interface SandboxView {
  id: string
  project: string
  name: string
  tier: string | null
  strategy: string | null
  state: string
  endpoint: { kind: string; addr?: string; scheme?: string } | null
  actors: string[]
  created_at: string
  idle_since: string | null
  has_owner: boolean
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const body = host.shadowRoot?.getElementById('body')
    if (!body) return

    if ((scope.autoFetch() ?? 'true') === 'false') return

    renderLoading(body)

    void fetchSandboxes()
      .then((sandboxes) => {
        host.setAttribute('count', String(sandboxes.length))
        if (sandboxes.length === 0) {
          renderEmpty(body)
        } else {
          renderGroups(body, sandboxes)
        }
      })
      .catch((err) => {
        renderError(body, err?.message ?? 'sandboxes request failed')
      })
  },
})

function renderLoading(target: HTMLElement) {
  target.replaceChildren(makeNode('div', { class: 'loading' }, 'Loading sandboxes…'))
}

function renderEmpty(target: HTMLElement) {
  target.replaceChildren(
    makeNode(
      'div',
      { class: 'empty' },
      'No sandboxes registered. They appear here when actors spawn.',
    ),
  )
}

function renderError(target: HTMLElement, message: string) {
  target.replaceChildren(makeNode('div', { class: 'error' }, `Error: ${message}`))
}

function renderGroups(target: HTMLElement, sandboxes: SandboxView[]) {
  const byProject = new Map<string, SandboxView[]>()
  for (const s of sandboxes) {
    const list = byProject.get(s.project) ?? []
    list.push(s)
    byProject.set(s.project, list)
  }
  const projects = Array.from(byProject.keys()).sort()
  const root = document.createDocumentFragment()
  for (const project of projects) {
    const sandboxesInProj = byProject.get(project)!
    const group = makeNode('div', { class: 'project' })
    group.append(makeNode('div', { class: 'project-name' }, displayProject(project)))
    const ul = makeNode('ul', {})
    for (const s of sandboxesInProj) {
      ul.append(renderSandboxRow(s))
    }
    group.append(ul)
    root.append(group)
  }
  target.replaceChildren(root)
}

function renderSandboxRow(s: SandboxView): HTMLElement {
  const nameSpan = makeNode('span', { class: 'name' }, s.name)
  nameSpan.append(makeNode('span', { class: 'id' }, s.id))
  return makeNode(
    'li',
    {},
    nameSpan,
    makeNode('span', { class: 'tier' }, s.tier ? `tier:${s.tier}` : 'tier:?'),
    makeNode('span', { class: 'state', 'data-state': s.state }, s.state),
    makeNode(
      'span',
      { class: 'actors' },
      s.actors.length === 0 ? 'idle' : `${s.actors.length} actor${s.actors.length === 1 ? '' : 's'}`,
    ),
  )
}

function displayProject(project: string): string {
  // Project keys are workspace paths today. Trim to last segment for
  // readability; full path stays visible on hover.
  const segments = project.split('/').filter(Boolean)
  return segments.at(-1) ?? project
}

async function fetchSandboxes(): Promise<SandboxView[]> {
  const resp = await fetch('/api/v1/sandboxes', {
    headers: { accept: 'application/json' },
  })
  if (!resp.ok) {
    throw new Error(`sandboxes endpoint returned ${resp.status}`)
  }
  const json = await resp.json()
  const raw = json?.data?.sandboxes
  if (!Array.isArray(raw)) return []
  return raw.map(coerce)
}

function coerce(raw: unknown): SandboxView {
  const r = (raw ?? {}) as Record<string, unknown>
  return {
    id: String(r.id ?? ''),
    project: String(r.project ?? ''),
    name: String(r.name ?? ''),
    tier: r.tier == null ? null : String(r.tier),
    strategy: r.strategy == null ? null : String(r.strategy),
    state: String(r.state ?? 'unknown'),
    endpoint: (r.endpoint as SandboxView['endpoint']) ?? null,
    actors: Array.isArray(r.actors) ? r.actors.map(String) : [],
    created_at: String(r.created_at ?? ''),
    idle_since: r.idle_since == null ? null : String(r.idle_since),
    has_owner: Boolean(r.has_owner),
  }
}

function makeNode(
  tag: string,
  attrs: Record<string, string> = {},
  ...children: (Node | string | null)[]
): HTMLElement {
  const el = document.createElement(tag)
  for (const [k, v] of Object.entries(attrs)) el.setAttribute(k, v)
  for (const c of children) {
    if (c == null) continue
    el.append(typeof c === 'string' ? document.createTextNode(c) : c)
  }
  return el
}
