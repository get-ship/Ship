import { chrome, controller, derived, signal } from '@ship/sui-runtime'
import type { ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-nav-top.sui'

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
  // Views are client-side routes; the daemon discovery catalog was quarantined
  // (06153c1a5). Cut the dead op — rail labels fall back to humanize() until the
  // router→view-list integration lands.
  return new Map<string, string>()
}

function navigateDoc(to: string): void {
  chrome.navigate(to)
}

export default controller({
  scope: (state: ViewScope<Scope>) => {
    const path = state.path
    const projectProp = state.project
    const viewProp = state.view
    const labels = signal<Map<string, string>>(new Map())
    const dockOpen = signal(false)

    const crumbs = derived<Crumb[]>(() => {
      const labelsMap = labels()
      const project = projectProp().trim()
      const view = viewProp().trim()

      // Source segments: explicit `view` prop overrides path entirely.
      // Otherwise split the path; empty path falls back to the "dashboard"
      // crumb (parity with ship-nav-top.renderCrumbs).
      const segs: string[] = view
        ? [view]
        : path().split('/').filter(Boolean)
      // Home (`/`) shows no view crumb — the brand logo is the home button.

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


    function onCrumbClick(e: MouseEvent, href: string): void {
      e.preventDefault()
      navigateDoc(href)
    }
    function onBrand(e: MouseEvent): void {
      e.preventDefault()
      navigateDoc('/')
    }
    function onFullscreen(): void {
      if (document.fullscreenElement) void document.exitFullscreen()
      else void document.documentElement.requestFullscreen()
    }

    return {
      labels, dockOpen, crumbs,
      onCrumbClick, onBrand, onFullscreen,
    }
  },

  mount(state, host) {
    const pathSig = state.$signals.path
    const labelsSig = state.$signals.labels
    const dockOpenSig = state.$signals.dockOpen

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
