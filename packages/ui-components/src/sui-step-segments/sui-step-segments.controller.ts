import { controller, effect } from '@ship/sui-runtime'
import type { Scope } from './sui-step-segments.sui'

const KNOWN = new Set(['done', 'failed', 'pending', 'running'])

function parseValues(raw: string): string[] {
  return raw
    .split(',')
    .map((s) => s.trim().toLowerCase())
    .filter((s) => s.length > 0)
    .map((s) => (KNOWN.has(s) ? s : 'pending'))
}

export default controller<Scope>({
  mount(state, _host) {
    const scope = state as unknown as Scope

    const stopEffect = effect(() => {
      const segments = parseValues(scope.values() || '')
      const row = scope.rowEl() as HTMLElement | null
      if (!row) return

      row.textContent = ''

      if (segments.length === 0) {
        row.removeAttribute('role')
        row.removeAttribute('aria-label')
        return
      }

      row.setAttribute('role', 'img')
      row.setAttribute(
        'aria-label',
        `${segments.length} step outcome${segments.length === 1 ? '' : 's'}`,
      )

      for (const segState of segments) {
        const seg = document.createElement('span')
        seg.className = 'seg'
        seg.setAttribute('part', 'seg')
        seg.setAttribute('data-state', segState)
        row.appendChild(seg)
      }
    })

    return () => stopEffect()
  },
})
