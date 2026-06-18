// <sui-command-bar> controller — the cmdk palette. Reads the actual
// RPC surface (ship.agents.summaries(), ship.pipelines.list()) to
// build the item catalog. Navigates via the public runtime API
// (@ship/deck/route's `navigate()`) — no hand-rolled
// `window.ship.navigate` or `chrome.navigate`.
//
// ⌘K opens the palette; esc closes; type to filter; enter or click
// activates; arrow keys move the selected row.

import { controller, signal, effect, derived, ref, shared, type ViewScope } from '@ship/sui-runtime'
import { navigate } from '@ship/deck/route'
import { getShipClient } from '@ship/sdk-client'
import type { Scope } from './sui-command-bar.sui'

interface CmdItem {
  id: string
  label: string
  kind: 'agent' | 'pipeline' | 'view'
  href?: string
}

interface AgentSummary {
  id?: string
  name?: string
  status?: string
}

interface PipelineIndexEntry {
  id?: string
  name?: string
  slug?: string
}

export default controller<Scope>({
  scope: (_ctx: ViewScope<Scope>) => {
    const ship = getShipClient()

    const query = shared(signal(''))
    const items = shared(signal<CmdItem[]>([]))
    const selectedIdx = shared(signal(0))
    const dialogRef = ref<HTMLElement>(null)

    const filteredItems = derived(() => {
      const q = query().trim().toLowerCase()
      const all = items()
      if (!q) return all
      return all.filter((it) => it.label.toLowerCase().includes(q) || it.kind.includes(q))
    })

    // Trigger label: the dialog is closed → show the search hint.
    const triggerLabel = derived(() => (query() ? query() : 'Search anything'))

    // Live data: agents and pipelines from the SDK.
    effect(() => {
      const out: CmdItem[] = []
      // Static view items — every chrome page is navigable.
      out.push({ id: 'v:home', label: 'Home', kind: 'view', href: '/' })
      out.push({ id: 'v:studio', label: 'Studio', kind: 'view', href: '/studio' })
      // Dynamic items come from the SDK.
      const agentsSrc = ship.agents.summaries()
      const applyAgents = (s: { items?: AgentSummary[] } | AgentSummary[] | undefined) => {
        const list = Array.isArray(s) ? s : s?.items ?? []
        for (const a of list) {
          const id = a.id ?? ''
          if (!id) continue
          out.push({
            id: `a:${id}`,
            label: a.name ?? id,
            kind: 'agent',
            href: `/studio/agents/${encodeURIComponent(id)}`,
          })
        }
      }
      const initialA = (agentsSrc as { value?: unknown }).value
      if (initialA !== undefined) applyAgents(initialA as Parameters<typeof applyAgents>[0])
      const subA = (agentsSrc as { subscribe?: (cb: (v: unknown) => void) => () => void }).subscribe
      const unsubA = typeof subA === 'function' ? subA((v) => { items.set(rebuild(out, applyAgents, v)) }) : undefined
      // Pipelines
      const pipesSrc = (ship as { pipelines?: { list: () => unknown } }).pipelines?.list()
      if (pipesSrc) {
        const applyPipes = (s: { items?: PipelineIndexEntry[] } | PipelineIndexEntry[] | undefined) => {
          const list = Array.isArray(s) ? s : s?.items ?? []
          for (const p of list) {
            const id = p.id ?? p.slug ?? ''
            if (!id) continue
            out.push({
              id: `p:${id}`,
              label: p.name ?? id,
              kind: 'pipeline',
              href: `/studio/pipelines/run/${encodeURIComponent(id)}`,
            })
          }
        }
        const initialP = (pipesSrc as { value?: unknown }).value
        if (initialP !== undefined) applyPipes(initialP as Parameters<typeof applyPipes>[0])
        const subP = (pipesSrc as { subscribe?: (cb: (v: unknown) => void) => () => void }).subscribe
        const unsubP = typeof subP === 'function' ? subP((v) => { items.set(rebuild(out, applyPipes, v)) }) : undefined
        return () => { unsubA?.(); unsubP?.() }
      }
      return unsubA
    })

    function rebuild<T>(out: CmdItem[], apply: (v: T) => void, v: unknown): CmdItem[] {
      // Apply replaces the dynamic tail; the head (static views) stays.
      const head = out.filter((it) => it.id.startsWith('v:'))
      const tail: CmdItem[] = []
      const save = out
      out.length = 0
      out.push(...head)
      apply(v as T)
      // After apply, `out` is refilled — capture the dynamic tail.
      const next = [...out]
      // Restore `out` to the saved list (mutated tail now lives in `next`).
      out.length = 0
      out.push(...save)
      // Replace the dynamic tail in `next` with the freshly applied ones.
      void tail
      return next
    }

    function openPalette(): void {
      // `open` is a `<prop>` on the template; the host reads the new
      // value and toggles the dialog. We can't write props from inside
      // the controller — the runtime wires the prop signal. We emit an
      // event the host listens to (or the host reads `this.open` and
      // toggles). For now: dispatch a CustomEvent the host toggles.
      _ctx?.host.dispatchEvent(new CustomEvent('sui-open-command-palette', { bubbles: true, composed: true }))
    }

    function selectItem(item: CmdItem): void {
      if (item.href) navigate(item.href)
      // Close the palette by emitting the close event.
      _ctx?.host.dispatchEvent(new CustomEvent('sui-close-command-palette', { bubbles: true, composed: true }))
    }

    function onInputKey(e: KeyboardEvent): void {
      const list = filteredItems()
      if (e.key === 'ArrowDown') {
        e.preventDefault()
        selectedIdx.set(Math.min(selectedIdx() + 1, list.length - 1))
      } else if (e.key === 'ArrowUp') {
        e.preventDefault()
        selectedIdx.set(Math.max(selectedIdx() - 1, 0))
      } else if (e.key === 'Enter') {
        e.preventDefault()
        const it = list[selectedIdx()]
        if (it) selectItem(it)
      } else if (e.key === 'Escape') {
        e.preventDefault()
        _ctx?.host.dispatchEvent(new CustomEvent('sui-close-command-palette', { bubbles: true, composed: true }))
      }
    }

    return { query, items, filteredItems, selectedIdx, dialogRef, openPalette, selectItem, onInputKey, triggerLabel }
  },
})
