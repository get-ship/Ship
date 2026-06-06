import { controller, signal, derived, effect, ref } from '@ship/sui-runtime'
import type { Signal, ViewScope } from '@ship/sui-runtime'
import type { Scope } from './sui-project-switcher.sui'
import { ops } from '../../ops'

interface Project {
  id: string
  name: string
  path: string
}

// Deterministic per-project identity hue — the daemon exposes no colour field,
// so derive one from the id (stable across reloads). Mirrors the palette in
// views/agents/[id]/hue.ts; consolidate into a shared util when agents folds
// into studio.
const HUES = ['orange', 'amber', 'red', 'violet', 'blue', 'teal', 'green', 'pink', 'slate'] as const
function pickHue(id: string): string {
  if (!id) return 'slate'
  let h = 0
  for (let i = 0; i < id.length; i++) h = (h * 31 + id.charCodeAt(i)) | 0
  return HUES[Math.abs(h) % HUES.length]
}

type PopoverEl = HTMLElement & { showPopover?: () => void; hidePopover?: () => void }

export default controller({
  scope: (_ctx: ViewScope<Scope>) => {
    const projects = signal<Project[]>([])
    const activeId = signal('')
    const menuOpen = signal(false)

    // Hue is stable per id; the active-row marking is computed reactively in
    // the template against activeId() so it updates when projects load async.
    const rows = derived(() => projects().map((p) => ({ ...p, hue: pickHue(p.id) })))
    const activeName = derived(() => projects().find((p) => p.id === activeId())?.name ?? 'Project')
    const activeHue = derived(() => pickHue(activeId()))
    const triggerAria = derived(() => `Active project: ${activeName()}. Switch project.`)

    const triggerBtn = ref<HTMLButtonElement>(null)
    const menuEl = ref<HTMLDivElement>(null)
    const internals: { menu: PopoverEl | null } = { menu: null }

    function toggleMenu(): void {
      const menu = internals.menu
      if (!menu) return
      if (menuOpen()) menu.hidePopover?.()
      else menu.showPopover?.()
    }

    async function loadProjects(): Promise<void> {
      try {
        const resp = await ops.space.list({})
        projects.set(resp.spaces.map((s) => ({ id: s.id, name: s.name, path: s.path })))
        if (resp.activeId) activeId.set(resp.activeId)
      } catch {
        // offline — leave the trigger showing the placeholder name
      }
    }

    async function pick(id: string): Promise<void> {
      internals.menu?.hidePopover?.()
      if (id === activeId()) return
      try {
        await ops.space.activate({ id })
        // Switching projects swaps the whole workspace; reload into the root so
        // every view re-resolves against the newly-active project.
        location.assign('/')
      } catch {
        // ignore — daemon unreachable
      }
    }

    return {
      projects,
      activeId,
      menuOpen,
      rows,
      activeName,
      activeHue,
      triggerAria,
      triggerBtn,
      menuEl,
      toggleMenu,
      pick,
      loadProjects,
      _internals: internals,
    }
  },

  mount(state, host) {
    const scope = state as unknown as {
      menuOpen: Signal<boolean>
      menuEl: () => PopoverEl | null
      triggerBtn: () => HTMLElement | null
      loadProjects: () => Promise<void>
      _internals: { menu: PopoverEl | null }
    }

    void scope.loadProjects()

    // Mirror the native popover open/close into menuOpen + anchor on open.
    const stopSync = effect(() => {
      const menu = scope.menuEl()
      scope._internals.menu = menu
      if (!menu) return
      const onToggle = (e: Event) => {
        const open = (e as ToggleEvent).newState === 'open'
        if (scope.menuOpen() !== open) scope.menuOpen.set(open)
        if (open) anchor()
      }
      menu.addEventListener('toggle', onToggle)
      return () => menu.removeEventListener('toggle', onToggle)
    })

    function anchor(): void {
      const menu = scope.menuEl()
      const trigger = scope.triggerBtn()
      if (!menu || !trigger) return
      const r = trigger.getBoundingClientRect()
      menu.style.top = `${r.bottom + 4}px`
      menu.style.left = `${Math.max(8, r.left)}px`
    }

    host.dispatchEvent(
      new CustomEvent('sui:project-switcher:ready', { bubbles: true, composed: true }),
    )

    return () => {
      stopSync()
    }
  },
})
