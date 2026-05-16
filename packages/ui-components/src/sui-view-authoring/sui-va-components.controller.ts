import { controller, signal, effect, chrome } from '@ship/sui-runtime'
import type { Scope } from './sui-va-components.sui'

interface ViewMcp {
  readSource(name: string): Promise<{ source: string }>
}

declare global {
  interface Window {
    __viewMcp?: ViewMcp
  }
}

// Read source via the chrome's view-MCP bridge, retrying once on
// failure to absorb the natural race between edit-mode/enter (grants
// capability) and the VA components' first auto-refresh.
async function readSourceWithRetry(mcp: ViewMcp, name: string): Promise<string> {
  try {
    const r = await mcp.readSource(name)
    return r.source
  } catch {
    await new Promise((r) => setTimeout(r, 200))
    const r = await mcp.readSource(name)
    return r.source
  }
}

interface ComponentRef { tag: string; count: number }

/** Cheap scan: count <sui-*> opening tags in the source string. The
 *  full parser lives in the Rust compiler — for the dev panel a regex
 *  pass is good enough to surface what's in use. */
function scanComponents(source: string): ComponentRef[] {
  const counts = new Map<string, number>()
  const re = /<(sui-[a-z][a-z0-9-]*)\b/g
  let m: RegExpExecArray | null
  while ((m = re.exec(source)) !== null) {
    const tag = m[1]!.toLowerCase()
    counts.set(tag, (counts.get(tag) ?? 0) + 1)
  }
  return Array.from(counts.entries())
    .map(([tag, count]) => ({ tag, count }))
    .sort((a, b) => a.tag.localeCompare(b.tag))
}

function highlightInMount(tag: string): void {
  // Pulse all instances of `tag` in the mounted view so the user can
  // see which DOM nodes back the chip they clicked. We tag them with a
  // data attribute that the shell stylesheet decorates, then remove
  // after a beat.
  const slot = document.getElementById('ship-content')
  if (!slot) return
  const matches = slot.querySelectorAll(tag)
  for (const el of matches) {
    (el as HTMLElement).setAttribute('data-va-highlight', '')
  }
  window.setTimeout(() => {
    for (const el of matches) (el as HTMLElement).removeAttribute('data-va-highlight')
  }, 1400)
}

export default controller<Scope>({
  scope: () => {
    const inView = signal<ComponentRef[]>([])
    const highlighted = signal<string>('')

    function onChipClick(tag: string): void {
      highlighted.set(tag)
      highlightInMount(tag)
      window.setTimeout(() => {
        if (highlighted() === tag) highlighted.set('')
      }, 1500)
    }

    function onPicked(e: Event): void {
      const tags = ((e as CustomEvent).detail?.tags ?? []) as string[]
      if (tags.length === 0) return
      // Hand off to the agent via the command bar in chat mode. The
      // prefill describes the desired action so the agent's planner can
      // pick it up directly.
      const list = tags.map((t) => `<${t}>`).join(', ')
      chrome.bar.open('ai', `Add ${list} to the current view.`)
    }

    return { inView, highlighted, onChipClick, onPicked }
  },

  mount(state) {
    const s = state as unknown as {
      inView: ReturnType<typeof signal<ComponentRef[]>>
      view: () => string
    }

    let loadedFor: string | null = null

    async function refresh(): Promise<void> {
      const name = s.view()
      if (!name) { s.inView.set([]); return }
      if (loadedFor === name) return
      loadedFor = name
      const mcp = window.__viewMcp
      if (!mcp) return
      try {
        const source = await readSourceWithRetry(mcp, name)
        s.inView.set(scanComponents(source))
      } catch {
        // Reset the guard so a subsequent view switch (or a manual
        // re-mount of the tab) can re-attempt — otherwise an early
        // failure leaves the chips section permanently empty.
        loadedFor = null
        s.inView.set([])
      }
    }

    const stopEffect = effect(() => {
      s.view() // track
      void refresh()
    })

    return () => stopEffect()
  },
})
