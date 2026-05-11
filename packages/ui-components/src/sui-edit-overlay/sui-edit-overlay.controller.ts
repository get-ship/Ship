import { controller } from '@ship/sui-runtime'
import type { Scope } from './sui-edit-overlay.sui'

interface Highlight {
  top: number
  left: number
  width: number
  height: number
  tag: string
  id: string | null
}

function targetFromEvent(e: PointerEvent | MouseEvent): HTMLElement | null {
  const root = document.getElementById('ship-view-root')
  if (!root) return null
  const target = e.target as HTMLElement | null
  if (!target || !root.contains(target)) return null
  if (target.closest('sui-edit-overlay')) return null
  return target
}

function describe(el: HTMLElement): { tag: string; selector: string; id: string | null } {
  const tag = el.tagName.toLowerCase()
  const id = el.id || null
  let selector = tag
  if (id) selector = `#${id}`
  else if (el.classList.length > 0) {
    selector = `${tag}.${Array.from(el.classList).slice(0, 2).join('.')}`
  }
  return { tag, selector, id }
}

export default controller<Scope>({
  mount(state, host) {
    const scope = state as unknown as Scope
    const boxEl = scope.boxEl() as HTMLElement | null
    const labelEl = scope.labelEl() as HTMLElement | null

    let _raf: number | null = null

    function applyHighlight(hl: Highlight | null) {
      if (!boxEl || !labelEl) return
      if (!hl) {
        boxEl.classList.remove('visible')
        labelEl.classList.remove('visible')
        return
      }
      boxEl.style.cssText = `top:${hl.top}px;left:${hl.left}px;width:${hl.width}px;height:${hl.height}px`
      boxEl.classList.add('visible')
      labelEl.style.cssText = `top:${hl.top}px;left:${hl.left}px`
      labelEl.textContent = `<${hl.tag}${hl.id ? `#${hl.id}` : ''}>`
      labelEl.classList.add('visible')
    }

    const onPointerMove = (e: PointerEvent): void => {
      if (_raf) return
      _raf = requestAnimationFrame(() => {
        _raf = null
        const target = targetFromEvent(e)
        if (!target) { applyHighlight(null); return }
        const rect = target.getBoundingClientRect()
        const { tag, id } = describe(target)
        applyHighlight({ top: rect.top, left: rect.left, width: rect.width, height: rect.height, tag, id })
      })
    }

    const onPointerLeave = (): void => {
      applyHighlight(null)
    }

    const onClick = (e: MouseEvent): void => {
      const target = targetFromEvent(e)
      if (!target) return
      e.preventDefault()
      e.stopPropagation()
      const { tag, selector, id } = describe(target)
      const shellEl = host.closest('sui-shell') ?? host.closest('ship-shell')
      shellEl?.dispatchEvent(
        new CustomEvent('sui:chat:annotate', {
          detail: {
            tag,
            selector,
            id,
            preview: (target.textContent ?? '').trim().slice(0, 60),
          },
          bubbles: true,
        }),
      )
      ;(window as any).ship?.chat?.open?.()
    }

    function setupListeners() {
      const root = document.getElementById('ship-view-root')
      if (!root) return
      root.addEventListener('pointermove', onPointerMove, true)
      root.addEventListener('pointerleave', onPointerLeave, true)
      root.addEventListener('click', onClick, true)
    }

    function teardownListeners() {
      const root = document.getElementById('ship-view-root')
      if (!root) return
      root.removeEventListener('pointermove', onPointerMove, true)
      root.removeEventListener('pointerleave', onPointerLeave, true)
      root.removeEventListener('click', onClick, true)
    }

    function applyActive() {
      const on = document.body.classList.contains('edit-mode')
      host.classList.toggle('active', on)
      if (on) setupListeners()
      else {
        teardownListeners()
        applyHighlight(null)
        if (_raf) { cancelAnimationFrame(_raf); _raf = null }
      }
    }

    applyActive()

    const observer = new MutationObserver(() => applyActive())
    observer.observe(document.body, { attributes: true, attributeFilter: ['class'] })

    return () => {
      observer.disconnect()
      teardownListeners()
      if (_raf) { cancelAnimationFrame(_raf); _raf = null }
    }
  },
})
