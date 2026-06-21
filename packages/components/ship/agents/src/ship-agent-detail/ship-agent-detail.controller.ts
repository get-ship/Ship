// <ship-agent-detail> — @ship/agents · the hub `/studio/agents/:id` page.
//
// Page-level detail composite. Owns the data wiring for a single
// agent: ship.agents.detail({ id }) via live() inside the controller;
// the hub route just supplies the route param. The detail surface
// composes the per-agent data into the standard detail-page layout
// (identity header, description, tags, capabilities, source path,
// start action, plus the back navigation).
//
// The `detail` payload is `unknown` from the SDK; the controller
// narrows to the same shape the existing <ship-agent-card> renders
// against (id, name, description, source, tags, harness, model,
// status, capabilities). When the daemon grows the payload (skills,
// mcp, rules, providers, permissions), the controller's narrow
// shape stays stable — the SDK op carries the additional fields
// silently; the rendering reads only what's declared.
//
// Hue + initials: prefer the values the agent doc saved (the
// `hue` + `letter` fields, same precedence the shell's detail page
// uses). Fall back to a deterministic id-hash so older agents still
// get a stable colour and a stable two-letter monogram.

import { controller, derived, live, unwrapOr } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import { ship } from '@ship/sdk-client'
import type { Scope } from './ship-agent-detail.sui'

interface Capability {
  label: string
  tier: string
}

interface AgentDetail {
  id: string
  name: string
  description: string
  source: string
  tags: string[]
  harness: string
  model: string
  status: string
  capabilities: Capability[]
  hue?: string
  letter?: string
}

const HUES = [
  'orange', 'amber', 'red', 'violet', 'blue',
  'teal', 'green', 'pink', 'slate',
] as const

const EMPTY: AgentDetail = {
  id: '', name: '', description: '', source: '', tags: [],
  harness: '', model: '', status: '', capabilities: [],
}

function pickHue(id: string): string {
  if (!id) return 'slate'
  let h = 0
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) | 0
  return HUES[Math.abs(h) % HUES.length]
}

function pickInitials(id: string, name?: string): string {
  const src = (id || name || '').trim()
  if (!src) return '?'
  const parts = src.split(/[-_\s]+/).filter(Boolean)
  if (parts.length >= 2) return (parts[0][0] + parts[1][0]).toUpperCase()
  const head = parts[0] || src
  return (head.length >= 2 ? head.slice(0, 2) : head).toUpperCase()
}

function isHue(s: string): boolean {
  for (const h of HUES) if (h === s) return true
  return false
}

export default controller({
  scope: ({ id }: ViewScope<Scope>, host?: Element) => {
    const detail = live(() => ship.agents.detail({ id: id() }))
    const data = derived<AgentDetail>(() => ({ ...EMPTY, ...(unwrapOr(detail, EMPTY) as Partial<AgentDetail>) }))

    const name = derived(() => data().name || data().id || id())
    const description = derived(() => {
      const d = data().description
      return d && d.trim().length > 0 ? d : 'No description provided.'
    })
    const source = derived(() => data().source || 'local')
    const tags = derived<string[]>(() => (Array.isArray(data().tags) ? data().tags : []))
    const capabilities = derived<Capability[]>(() => (Array.isArray(data().capabilities) ? data().capabilities : []))
    const harness = derived(() => data().harness)
    const model = derived(() => data().model)
    const status = derived(() => data().status)

    // Hue: prefer the value the agent doc saved (matches the create
    // flow's chosen colour), fall back to the id-hash. Validate
    // against the palette so a stale value doesn't leak through.
    const hue = derived(() => {
      const a = data()
      const saved = typeof a.hue === 'string' ? a.hue.trim() : ''
      if (saved.length > 0 && isHue(saved)) return saved
      return pickHue(a.id || id())
    })

    const initials = derived(() => {
      const a = data()
      const letter = typeof a.letter === 'string' ? a.letter.trim() : ''
      if (letter.length > 0) return letter.slice(0, 2).toUpperCase()
      return pickInitials(a.id || id(), a.name)
    })

    const isActive = derived(() => data().status === 'active' || data().status === 'busy')
    const statusLabel = derived(() => {
      const s = status()
      return s ? s : 'idle'
    })

    // Friendly source-path hint for the metadata row. The SDK doesn't
    // surface a literal on-disk path; we hint the convention so the
    // user can find the doc in the workspace tree.
    const sourcePath = derived(() => `.ship/agents/${data().id || id()}.jsonc`)

    const onBack = (e: MouseEvent): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-agent-back', {
          bubbles: true,
          composed: true,
        }),
      )
    }

    const onStart = (e: MouseEvent): void => {
      e.preventDefault()
      host?.dispatchEvent(
        new CustomEvent('ship-agent-start', {
          detail: { id: data().id || id() },
          bubbles: true,
          composed: true,
        }),
      )
    }

    return {
      detail, data, name, description, source, tags, capabilities,
      harness, model, status, hue, initials,
      isActive, statusLabel, sourcePath,
      onBack, onStart,
    }
  },
})

// Re-export the pure helpers so a unit test can exercise the
// shape-narrower and the hue/initials derivation without spinning up
// the controller.
export const __test__ = { pickHue, pickInitials, isHue, HUES }
