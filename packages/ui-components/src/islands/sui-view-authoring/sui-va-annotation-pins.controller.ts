import { controller, signal, chrome } from '@ship/sui-runtime'
import type { Scope } from './sui-va-annotation-pins.sui'

interface Pin {
  id: string
  number: number
  selector: string
  label: string
  turn: number
  x: number
  y: number
  fading: boolean
}

const AUTO_CLEAR_MS = 30_000

export default controller<Scope>({
  scope: () => {
    const pins = signal<Pin[]>([])
    function onPinClick(pin: Pin): void {
      chrome.dock.show()
      chrome.dock.activate('va-activity')
      document.dispatchEvent(new CustomEvent('sui:va:scroll-activity', {
        detail: { pinId: pin.id, selector: pin.selector },
        bubbles: true,
      }))
    }
    return { pins, onPinClick }
  },

  mount(state, host) {
    const s = state as unknown as { pins: ReturnType<typeof signal<Pin[]>> }
    let counter = 0
    let lastTurn = 0
    const timers = new Map<string, number>()

    function locate(selector: string): { x: number; y: number } | null {
      try {
        const slot = document.getElementById('ship-content')
        if (!slot) return null
        const el = slot.querySelector(selector) as HTMLElement | null
        if (!el) return null
        const r = el.getBoundingClientRect()
        if (r.width === 0 && r.height === 0) return null
        return { x: r.left + r.width / 2, y: r.top }
      } catch { return null }
    }

    function removePin(id: string): void {
      const next = s.pins().filter((p) => p.id !== id)
      s.pins.set(next)
      const tid = timers.get(id)
      if (tid != null) { window.clearTimeout(tid); timers.delete(id) }
    }

    function startFade(id: string): void {
      const next = s.pins().map((p) => p.id === id ? { ...p, fading: true } : p)
      s.pins.set(next)
      window.setTimeout(() => removePin(id), 260)
    }

    function clearAllExceptTurn(turn: number): void {
      for (const p of s.pins()) {
        if (p.turn < turn) startFade(p.id)
      }
    }

    function onAnnotation(e: Event): void {
      const d = (e as CustomEvent).detail as {
        id?: string
        selector?: string
        label?: string
        turn?: number
        view?: string
      } | undefined
      if (!d || !d.selector) return
      const turn = d.turn ?? (lastTurn + 1)
      if (turn > lastTurn) {
        clearAllExceptTurn(turn)
        lastTurn = turn
      }
      const at = locate(d.selector)
      if (!at) return // selector not in DOM — silently skip
      counter += 1
      const id = d.id ?? `pin-${counter}`
      const pin: Pin = {
        id,
        number: counter,
        selector: d.selector,
        label: d.label ?? d.selector,
        turn,
        x: at.x, y: at.y,
        fading: false,
      }
      s.pins.set([...s.pins().filter((p) => p.id !== id), pin])
      const tid = window.setTimeout(() => startFade(id), AUTO_CLEAR_MS)
      timers.set(id, tid)
    }

    function reposition(): void {
      const cur = s.pins()
      if (cur.length === 0) return
      const next: Pin[] = []
      for (const p of cur) {
        const at = locate(p.selector)
        if (!at) continue
        next.push({ ...p, x: at.x, y: at.y })
      }
      s.pins.set(next)
    }

    document.addEventListener('sui:view:annotation', onAnnotation)
    window.addEventListener('resize', reposition)
    window.addEventListener('scroll', reposition, true)

    // Repositions when the mounted view changes (DOM swap).
    document.addEventListener('sui:scope:change', reposition)

    // Public API for tests: synchronously read pin count.
    Object.defineProperty(host, 'pinCount', {
      configurable: true,
      get: () => s.pins().filter((p) => !p.fading).length,
    })
    Object.defineProperty(host, 'clear', {
      configurable: true,
      value: () => { for (const p of s.pins()) startFade(p.id) },
    })

    return () => {
      document.removeEventListener('sui:view:annotation', onAnnotation)
      window.removeEventListener('resize', reposition)
      window.removeEventListener('scroll', reposition, true)
      document.removeEventListener('sui:scope:change', reposition)
      for (const tid of timers.values()) window.clearTimeout(tid)
      timers.clear()
    }
  },
})
