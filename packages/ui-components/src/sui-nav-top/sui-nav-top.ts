import { controller, effect, signal } from '@ship/ui-runtime'
import type { Scope } from './sui-nav-top.sui'

interface ApiView {
  name: string
  rail?: { label: string; icon: string }
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

function renderCrumbs(
  path: string,
  labels: Map<string, string>,
  container: HTMLElement,
): void {
  container.innerHTML = ''
  const segs = path.split('/').filter(Boolean)
  if (segs.length === 0) segs.push('dashboard')
  for (let i = 0; i < segs.length; i++) {
    const seg = segs[i]!
    const isLast = i === segs.length - 1
    const href = '/' + segs.slice(0, i + 1).join('/')
    const label = i === 0 ? (labels.get(seg) ?? humanize(seg)) : seg
    if (isLast) {
      const span = document.createElement('span')
      span.className = 'current'
      span.id = 'breadcrumbs-title'
      span.textContent = label
      container.appendChild(span)
    } else {
      const a = document.createElement('a')
      a.href = href
      a.textContent = label
      a.addEventListener('click', (e: MouseEvent) => {
        e.preventDefault()
        history.pushState({}, '', href)
        document.dispatchEvent(
          new CustomEvent('sui:navigate', { detail: { to: href } }),
        )
      })
      container.appendChild(a)
      const chevron = document.createElement('span')
      chevron.className = 'chevron'
      chevron.setAttribute('aria-hidden', 'true')
      chevron.textContent = '›'
      container.appendChild(chevron)
    }
  }
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    const labels = signal<Map<string, string>>(new Map())
    const dockOpen = signal(document.body.classList.contains('dock-open'))

    // Set initial path from location before registering effects.
    if (!scope.path()) scope.path.set(location.pathname)

    // ── reactive breadcrumb render ─────────────────────────────
    const stopCrumbs = effect(() => {
      const path = scope.path()
      const labelsMap = labels()
      const container = scope.breadcrumbsEl() as HTMLElement | null
      if (!container) return
      renderCrumbs(path, labelsMap, container)
    })

    // ── dock-open aria-pressed sync ────────────────────────────
    const stopDock = effect(() => {
      const open = dockOpen()
      const btn = scope.dockToggleBtn() as HTMLElement | null
      btn?.setAttribute('aria-pressed', open ? 'true' : 'false')
    })

    // ── fetch view labels (updates crumbs once resolved) ───────
    void fetchViewLabels().then((map) => labels.set(map))

    // ── navigation listener ────────────────────────────────────
    const onNavigate = (e: Event) => {
      const to = (e as CustomEvent<{ to?: string }>).detail?.to
      if (typeof to === 'string') scope.path.set(to)
    }
    document.addEventListener('sui:navigate', onNavigate)

    // ── dock body class observer ───────────────────────────────
    const dockObserver = new MutationObserver(() => {
      dockOpen.set(document.body.classList.contains('dock-open'))
    })
    dockObserver.observe(document.body, {
      attributes: true,
      attributeFilter: ['class'],
    })

    // ── click handlers ─────────────────────────────────────────
    const brandLink = scope.brandLink() as HTMLAnchorElement | null
    const railToggleBtn = scope.railToggleBtn() as HTMLElement | null
    const studioBtn = scope.studioBtn() as HTMLElement | null
    const dockToggleBtn = scope.dockToggleBtn() as HTMLElement | null
    const fullscreenBtn = scope.fullscreenBtn() as HTMLElement | null
    const chatToggleBtn = scope.chatToggleBtn() as HTMLElement | null

    const onBrand = (e: MouseEvent) => {
      e.preventDefault()
      history.pushState({}, '', '/')
      document.dispatchEvent(
        new CustomEvent('sui:navigate', { detail: { to: '/' } }),
      )
    }
    const onRailToggle = () =>
      host.dispatchEvent(
        new CustomEvent('sui:rail:toggle', { bubbles: true, composed: true }),
      )
    const onStudio = () => {
      history.pushState({}, '', '/ship-ui-studio')
      document.dispatchEvent(
        new CustomEvent('sui:navigate', { detail: { to: '/ship-ui-studio' } }),
      )
    }
    const onDockToggle = () =>
      host.dispatchEvent(
        new CustomEvent('sui:dock:toggle', { bubbles: true, composed: true }),
      )
    const onFullscreen = () => {
      if (document.fullscreenElement) void document.exitFullscreen()
      else void document.documentElement.requestFullscreen()
    }
    const onChatToggle = () =>
      host.dispatchEvent(
        new CustomEvent('sui:chat:toggle', { bubbles: true, composed: true }),
      )

    brandLink?.addEventListener('click', onBrand)
    railToggleBtn?.addEventListener('click', onRailToggle)
    studioBtn?.addEventListener('click', onStudio)
    dockToggleBtn?.addEventListener('click', onDockToggle)
    fullscreenBtn?.addEventListener('click', onFullscreen)
    chatToggleBtn?.addEventListener('click', onChatToggle)

    host.dispatchEvent(
      new CustomEvent('sui:nav-top:ready', { bubbles: true, composed: true }),
    )

    return () => {
      stopCrumbs()
      stopDock()
      document.removeEventListener('sui:navigate', onNavigate)
      dockObserver.disconnect()
    }
  },
})
