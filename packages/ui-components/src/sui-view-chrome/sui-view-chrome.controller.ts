import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-view-chrome.sui'

type SlotName = 'actions' | 'status-start' | 'status-end'

const SLOT_TARGETS: Record<SlotName, string> = {
  'actions': '#view-topbar-actions',
  'status-start': '#view-status-start',
  'status-end': '#view-status-end',
}

const ATTR_TARGETS = {
  title: '#breadcrumbs-title',
  branch: '#status-branch',
} as const

export default controller<Scope>({
  mount(state, host) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const scope = state as unknown as any
    const moved: Partial<Record<SlotName, HTMLElement[]>> = {}

    function contribute() {
      for (const name of Object.keys(SLOT_TARGETS) as SlotName[]) {
        const target = document.querySelector<HTMLElement>(SLOT_TARGETS[name])
        if (!target) continue
        const sourced = Array.from(host.querySelectorAll<HTMLElement>(`[slot="${name}"]`))
        if (sourced.length === 0) continue
        for (const node of sourced) target.appendChild(node)
        moved[name] = sourced
      }
    }

    function clearAttrs() {
      const titleEl = document.querySelector<HTMLElement>(ATTR_TARGETS.title)
      if (titleEl) titleEl.textContent = ''

      if (scope.branch()) {
        const branchEl = document.querySelector<HTMLElement>(ATTR_TARGETS.branch)
        if (branchEl) branchEl.textContent = ''
      }

      if (scope.ahead() > 0 || scope.behind() > 0) {
        const abEl = document.querySelector<HTMLElement>('#status-ab')
        if (abEl) abEl.textContent = ''
      }

      if (scope.tone() !== 'default' || scope.busy()) {
        const dot = document.querySelector<HTMLElement>('#status-dot')
        if (dot) {
          delete dot.dataset['tone']
          dot.removeAttribute('data-busy')
        }
      }
    }

    function withdraw() {
      for (const name of Object.keys(moved) as SlotName[]) {
        for (const n of moved[name] ?? []) n.remove()
        delete moved[name]
      }
      clearAttrs()
    }

    const stopAttrs = effect(() => {
      const title: string = scope.title()
      const branch: string = scope.branch()
      const ahead: number = scope.ahead()
      const behind: number = scope.behind()
      const tone: string = scope.tone()
      const busy: boolean = scope.busy()

      if (title) {
        const el = document.querySelector<HTMLElement>(ATTR_TARGETS.title)
        if (el) el.textContent = title
      }

      if (branch) {
        const el = document.querySelector<HTMLElement>(ATTR_TARGETS.branch)
        if (el) el.textContent = branch
      }

      if (ahead > 0 || behind > 0) {
        const el = document.querySelector<HTMLElement>('#status-ab')
        if (el) {
          const parts: string[] = []
          if (ahead > 0) parts.push(`↑${ahead}`)
          if (behind > 0) parts.push(`↓${behind}`)
          el.textContent = parts.join(' ')
        }
      }

      if (tone !== 'default' || busy) {
        const dot = document.querySelector<HTMLElement>('#status-dot')
        if (dot) {
          dot.dataset['tone'] = tone
          if (busy) dot.setAttribute('data-busy', '')
          else dot.removeAttribute('data-busy')
        }
      }
    })

    queueMicrotask(() => contribute())

    return () => {
      stopAttrs()
      withdraw()
    }
  },
})
