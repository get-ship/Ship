import { chrome, controller, derived, signal } from '@ship/sui-runtime'
import type { Signal } from '@ship/sui-runtime'
import type { Scope } from './sui-nav-top.sui'

interface ApiView {
  name: string
  rail?: { label: string; icon: string }
}

interface Crumb {
  href: string
  label: string
  isLast: boolean
  // The keyed iteration in the template includes `isLast` in the key so
  // SUI doesn't reuse a previous crumb's branch (current-vs-link) when
  // the user navigates to a deeper route. Pure-`href` keys re-use a
  // crumb that flipped from leaf to mid-path and the runtime keeps the
  // old `{#if}` branch mounted, leaving the breadcrumb missing its
  // chevron + link affordance on the formerly-leaf segment.
  key: string
}

function humanize(id: string): string {
  return id
    .replace(/-/g, ' ')
    .split(' ')
    .map((w) => (w ? w[0]!.toUpperCase() + w.slice(1) : w))
    .join(' ')
}

async function fetchViewLabels(): Promise<Map<string, string>> {
  try {
    const resp = await fetch('/api/v1/views', { credentials: 'same-origin' })
    if (!resp.ok) return new Map()
    const body = (await resp.json()) as { data?: { views?: ApiView[] } }
    const map = new Map<string, string>()
    for (const v of body.data?.views ?? []) {
      if (v.rail?.label) map.set(v.name, v.rail.label)
    }
    return map
  } catch {
    return new Map()
  }
}

function navigateDoc(to: string): void {
  chrome.navigate(to)
}

export default controller<Scope>({
  scope: (state, host) => {
    const path = (state as { path: () => string }).path
    const projectProp = (state as { project: () => string }).project
    const viewProp = (state as { view: () => string }).view
    const labels = signal<Map<string, string>>(new Map())
    const dockOpen = signal(false)

    const crumbs = derived<Crumb[]>(() => {
      const labelsMap = labels()
      const project = projectProp().trim()
      const view = viewProp().trim()

      // Source segments: explicit `view` prop overrides path entirely.
      // Otherwise split the path; empty path falls back to the "dashboard"
      // crumb (parity with ship-nav-top.renderCrumbs).
      const segs = view
        ? [view]
        : path().split('/').filter(Boolean)
      if (segs.length === 0) segs.push('dashboard')

      const out: Crumb[] = []
      if (project) {
        // Project segment links to root; never the leaf (a view always
        // follows). Matches the ship-shell home-link convention.
        out.push({ href: '/', label: project, isLast: false, key: '/|0' })
      }
      segs.forEach((seg, i) => {
        const isLast = i === segs.length - 1
        const href = '/' + segs.slice(0, i + 1).join('/')
        const label = i === 0 ? (labelsMap.get(seg) ?? humanize(seg)) : seg
        out.push({ href, label, isLast, key: `${href}|${isLast ? 1 : 0}` })
      })
      return out
    })

    const hostEl = host as HTMLElement | null

    function onCrumbClick(e: MouseEvent, href: string): void {
      e.preventDefault()
      navigateDoc(href)
    }
    function onBrand(e: MouseEvent): void {
      e.preventDefault()
      navigateDoc('/')
    }
    function onRailToggle(): void {
      hostEl?.dispatchEvent(
        new CustomEvent('sui:rail:toggle', { bubbles: true, composed: true }),
      )
    }
    function onDockToggle(): void {
      chrome.dock.toggle()
    }
    function onFullscreen(): void {
      if (document.fullscreenElement) void document.exitFullscreen()
      else void document.documentElement.requestFullscreen()
    }

    return {
      labels, dockOpen, crumbs,
      onCrumbClick, onBrand, onRailToggle,
      onDockToggle, onFullscreen,
    }
  },

  mount(state, host) {
    const scope = state as unknown as Scope
    const pathSig = (scope as unknown as { path: Signal<string> }).path
    const labelsSig = (scope as unknown as { labels: Signal<Map<string, string>> }).labels
    const dockOpenSig = (scope as unknown as { dockOpen: Signal<boolean> }).dockOpen

    if (!pathSig()) pathSig.set(location.pathname)
    dockOpenSig.set(document.body.classList.contains('dock-open'))

    void fetchViewLabels().then((map) => labelsSig.set(map))

    const onNavigate = (e: Event) => {
      const to = (e as CustomEvent<{ to?: string }>).detail?.to
      if (typeof to === 'string') pathSig.set(to)
    }
    document.addEventListener('sui:navigate', onNavigate)

    const dockObserver = new MutationObserver(() => {
      dockOpenSig.set(document.body.classList.contains('dock-open'))
    })
    dockObserver.observe(document.body, {
      attributes: true,
      attributeFilter: ['class'],
    })

    host.dispatchEvent(
      new CustomEvent('sui:nav-top:ready', { bubbles: true, composed: true }),
    )

    return () => {
      document.removeEventListener('sui:navigate', onNavigate)
      dockObserver.disconnect()
    }
  },
})
