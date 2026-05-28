// Action-mapped input: keyboard now, gamepad-button polling on demand.
// Bind keys (or gamepad-button labels) to a logical action name; the bus emits
// 'start' on the action's first-down frame and 'end' on its last-up frame.
// `repeat: false` only fires 'start' on rising edge; with `repeat: true` the
// caller can poll `isDown(action)` each tick.

export type Phase = 'start' | 'end'

export interface InputBus {
  bind(keys: string[], action: string, opts?: { repeat?: boolean }): () => void
  on(action: string, handler: (phase: Phase, detail?: unknown) => void): () => void
  isDown(action: string): boolean
  fire(action: string, detail?: unknown): void
  pollGamepads(): void
  destroy(): void
}

interface Binding {
  keys: Set<string>
  action: string
  repeat: boolean
  active: boolean
}

const KEY_ALIASES: Record<string, string> = {
  Space: ' ',
  Esc: 'Escape',
  Enter: 'Enter',
  Up: 'ArrowUp',
  Down: 'ArrowDown',
  Left: 'ArrowLeft',
  Right: 'ArrowRight',
}

const PREVENT_DEFAULT_KEYS = new Set([' ', 'ArrowUp', 'ArrowDown', 'ArrowLeft', 'ArrowRight'])

function normalizeKey(raw: string): string {
  const v = KEY_ALIASES[raw] ?? raw
  return v.length === 1 ? v.toLowerCase() : v
}

function isTextInputFocused(): boolean {
  const el = (typeof document !== 'undefined' ? document.activeElement : null) as HTMLElement | null
  if (!el) return false
  const tag = el.tagName
  if (tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT') return true
  if (el.isContentEditable) return true
  return false
}

export function createInput(host: HTMLElement | Window): InputBus {
  const bindings: Binding[] = []
  const subs = new Map<string, Set<(phase: Phase, detail?: unknown) => void>>()
  const downKeys = new Set<string>()
  const downGpButtons = new Set<string>()

  function fire(action: string, phase: Phase, detail?: unknown) {
    const set = subs.get(action)
    if (!set) return
    for (const fn of set) fn(phase, detail)
  }

  function evaluate() {
    for (const b of bindings) {
      let pressed = false
      for (const k of b.keys) {
        if (downKeys.has(k) || downGpButtons.has(k)) { pressed = true; break }
      }
      if (pressed && !b.active) { b.active = true; fire(b.action, 'start') }
      else if (!pressed && b.active) { b.active = false; fire(b.action, 'end') }
      else if (pressed && b.active && b.repeat) fire(b.action, 'start')
    }
  }

  const target = host as EventTarget
  function onKey(e: Event) {
    const ke = e as KeyboardEvent
    if (ke.repeat) return
    if (isTextInputFocused()) return
    const key = normalizeKey(ke.key)
    if (PREVENT_DEFAULT_KEYS.has(key)) ke.preventDefault()
    if (ke.type === 'keydown') downKeys.add(key)
    else downKeys.delete(key)
    evaluate()
  }
  target.addEventListener('keydown', onKey)
  target.addEventListener('keyup', onKey)

  return {
    bind(keys, action, opts) {
      const b: Binding = {
        keys: new Set(keys.map(normalizeKey)),
        action,
        repeat: opts?.repeat ?? true,
        active: false,
      }
      bindings.push(b)
      return () => {
        const i = bindings.indexOf(b)
        if (i >= 0) bindings.splice(i, 1)
      }
    },
    on(action, handler) {
      let set = subs.get(action)
      if (!set) { set = new Set(); subs.set(action, set) }
      set.add(handler)
      return () => { set!.delete(handler) }
    },
    isDown(action) {
      const b = bindings.find((x) => x.action === action)
      return b?.active ?? false
    },
    fire(action, detail) { fire(action, 'start', detail) },
    pollGamepads() {
      if (typeof navigator === 'undefined' || !navigator.getGamepads) return
      const prev = new Set(downGpButtons)
      downGpButtons.clear()
      const pads = navigator.getGamepads()
      for (const pad of pads) {
        if (!pad) continue
        pad.buttons.forEach((btn, i) => {
          if (btn.pressed) downGpButtons.add(`GamepadButton${i}`)
        })
      }
      if (prev.size !== downGpButtons.size || [...prev].some((k) => !downGpButtons.has(k))) {
        evaluate()
      }
    },
    destroy() {
      target.removeEventListener('keydown', onKey)
      target.removeEventListener('keyup', onKey)
      bindings.length = 0
      subs.clear()
      downKeys.clear()
      downGpButtons.clear()
    },
  }
}
