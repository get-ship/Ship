import { controller } from '@ship/ui-runtime'
import {
  THEMES,
  applyTheme,
  persistTheme,
  readStoredTheme,
  isValidTheme,
  type ThemeId,
  type Theme,
} from '../../../ship-set/src/themes'
import type { Scope } from './sui-theme-picker.sui'

// Group cards by personality. Mirrors apps/shell/src/main.ts mountThemePicker
// so the SUI port presents the catalog the same way the legacy shell does.
type Group = { label: string; ids: ReadonlyArray<ThemeId> }
const GROUPS: Group[] = [
  { label: 'Default', ids: ['ship-light', 'ship-dark'] },
  { label: 'Texture', ids: ['paper', 'halo'] },
  { label: 'Effects', ids: ['glass'] },
  { label: 'Special', ids: ['console', 'contrast'] },
]

const THEMES_BY_ID = new Map<ThemeId, Theme>(THEMES.map((t) => [t.id, t]))

// Render the inner card markup for a single theme. The wrapping <button>
// is created in the controller so we can attach event listeners to live
// element refs rather than re-querying after innerHTML.
function cardInner(t: Theme): string {
  return `
    <span class="tc-bg" data-theme-preview="${t.id}" aria-hidden="true">
      <span class="tc-overlay"></span>
      <span class="tc-card">
        <span class="tc-card-head">
          <span class="tc-eyebrow">THEME</span>
          <span class="tc-heading">${t.label}</span>
        </span>
        <span class="tc-typo">
          <span class="tc-body">The quick brown fox</span>
          <span class="tc-mono">{ build: ok }</span>
        </span>
        <span class="tc-controls">
          <span class="tc-cta">Action</span>
          <span class="tc-ghost">Cancel</span>
          <span class="tc-chip tc-chip-accent">New</span>
          <span class="tc-chip tc-chip-muted">Idle</span>
        </span>
      </span>
    </span>
    <span class="row">
      <span class="name">${t.label}</span>
      <span class="radio" aria-hidden="true"></span>
    </span>`
}

// Inject view-tokens.css (which the daemon bundles with themes.css) into
// the picker's shadow root so [data-theme-preview="X"] rules apply to
// cards in shadow. Without this, document-level rules can't reach shadow
// children.
function ensureThemesCssInShadow(host: HTMLElement) {
  const root = host.shadowRoot
  if (!root) return
  // One link per shadow root; component instances reuse the same href so
  // the browser HTTP cache hits after the first fetch on the page.
  if (root.querySelector('link[data-sui-theme-preview-styles]')) return
  const link = document.createElement('link')
  link.rel = 'stylesheet'
  link.href = '/api/assets/view-tokens.css'
  link.setAttribute('data-sui-theme-preview-styles', '')
  root.appendChild(link)
}

interface PopoverApi extends HTMLElement {
  show?: () => void
  hide?: () => void
  toggle?: () => void
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope

    ensureThemesCssInShadow(host)

    const popover = scope.popover() as PopoverApi | null
    const triggerBtn = scope.triggerBtn() as HTMLButtonElement | null
    const dotEl = scope.dotEl() as HTMLElement | null
    const labelEl = scope.labelEl() as HTMLElement | null
    const gridEl = scope.gridEl() as HTMLElement | null
    if (!gridEl || !triggerBtn) return

    // Build the grid programmatically. The catalog is JS data so this
    // would otherwise live as a {#each} loop — SUI templates don't support
    // iteration today, so the controller emits the markup once and
    // attaches listeners directly to live element refs.
    const cardEls: HTMLButtonElement[] = []
    const fragment = document.createDocumentFragment()
    for (const g of GROUPS) {
      const section = document.createElement('section')
      section.className = 'group'
      section.setAttribute('aria-label', g.label)
      const head = document.createElement('div')
      head.className = 'group-h'
      head.textContent = g.label
      section.appendChild(head)
      const rows = document.createElement('div')
      rows.className = 'group-rows'
      for (const id of g.ids) {
        const t = THEMES_BY_ID.get(id)
        if (!t) continue
        const card = document.createElement('button')
        card.type = 'button'
        card.className = 'card'
        card.setAttribute('role', 'option')
        card.dataset.themeCard = id
        card.setAttribute('aria-label', t.label)
        card.setAttribute('aria-selected', 'false')
        card.tabIndex = -1
        card.innerHTML = cardInner(t)
        rows.appendChild(card)
        cardEls.push(card)
      }
      section.appendChild(rows)
      fragment.appendChild(section)
    }
    gridEl.replaceChildren(fragment)

    function setActive(id: ThemeId) {
      for (const c of cardEls) {
        const on = c.dataset.themeCard === id
        c.setAttribute('aria-selected', on ? 'true' : 'false')
        c.tabIndex = on ? 0 : -1
      }
      paintTrigger(id)
    }

    function paintTrigger(id: ThemeId) {
      const t = THEMES_BY_ID.get(id)
      if (!t) return
      if (labelEl) labelEl.textContent = t.label
      if (dotEl) {
        // Swatches: [bg, card, primary, accent]. Primary drives the dot
        // since that's the brand-defining color in each theme.
        dotEl.style.background = t.swatches[2]
      }
    }

    // ── Initial paint ──────────────────────────────────────────────────
    const initial = readStoredTheme()
    // applyTheme is idempotent — if the page boot script already applied
    // the stored theme, this is a no-op except for the dispatch.
    applyTheme(initial)
    setActive(initial)

    // ── Click handler ──────────────────────────────────────────────────
    function onCardClick(card: HTMLButtonElement) {
      const id = card.dataset.themeCard
      if (!isValidTheme(id)) return
      const next = applyTheme(id)
      if (scope.persist()) persistTheme(next)
      setActive(next)
      // Bridge canonical ship event to a SUI-namespaced event for
      // listeners that prefer to track a single namespace.
      host.dispatchEvent(new CustomEvent('sui-theme-change', {
        detail: { id: next, dark: THEMES_BY_ID.get(next)?.dark ?? false },
        bubbles: true,
        composed: true,
      }))
      popover?.hide?.()
    }
    function onGridClick(e: Event) {
      const card = (e.target as HTMLElement).closest<HTMLElement>('[data-theme-card]')
      if (card && card instanceof HTMLButtonElement) onCardClick(card)
    }
    gridEl.addEventListener('click', onGridClick)

    // ── Keyboard nav across groups ─────────────────────────────────────
    function onGridKeydown(e: KeyboardEvent) {
      const idx = cardEls.findIndex((c) => c === gridEl?.getRootNode().activeElement)
      const move = (delta: number) => {
        const next = cardEls[(idx + delta + cardEls.length) % cardEls.length]
        if (!next) return
        for (const c of cardEls) c.tabIndex = -1
        next.tabIndex = 0
        next.focus()
        e.preventDefault()
      }
      switch (e.key) {
        case 'ArrowDown': move(1); break
        case 'ArrowUp':   move(-1); break
        case 'Home':      cardEls[0]?.focus(); e.preventDefault(); break
        case 'End':       cardEls[cardEls.length - 1]?.focus(); e.preventDefault(); break
        case 'Enter':
        case ' ':
          (gridEl?.getRootNode() as Document | ShadowRoot).activeElement instanceof HTMLButtonElement
            && (gridEl?.getRootNode() as Document | ShadowRoot).activeElement?.dispatchEvent(new MouseEvent('click', { bubbles: true }))
          e.preventDefault()
          break
      }
    }
    gridEl.addEventListener('keydown', onGridKeydown)

    // ── External theme changes → repaint trigger + selection ───────────
    // applyTheme dispatches `ship:theme:change` on document; listen so
    // settings views, devtools, or programmatic callers stay in sync.
    function onExternalChange(e: Event) {
      const detail = (e as CustomEvent<{ id?: string }>).detail
      const id = detail?.id
      if (!isValidTheme(id)) return
      // Avoid feedback loop: setActive does no DOM work if state matches.
      setActive(id)
    }
    document.addEventListener('ship:theme:change', onExternalChange)

    // Focus the first card when the popover opens — keyboard users
    // expect a starting position.
    function onPopoverToggle(e: Event) {
      const open = (e as CustomEvent<{ open: boolean }>).detail?.open
      if (!open) return
      // Defer to next frame so the popover is laid out before we focus.
      requestAnimationFrame(() => {
        const current = cardEls.find((c) => c.getAttribute('aria-selected') === 'true')
        ;(current ?? cardEls[0])?.focus()
      })
    }
    popover?.addEventListener('sui-toggle', onPopoverToggle)

    return () => {
      gridEl.removeEventListener('click', onGridClick)
      gridEl.removeEventListener('keydown', onGridKeydown)
      document.removeEventListener('ship:theme:change', onExternalChange)
      popover?.removeEventListener('sui-toggle', onPopoverToggle)
    }
  },
})
