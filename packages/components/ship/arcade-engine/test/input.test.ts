import { afterEach, describe, expect, it, vi } from 'vitest'
import { createInput, type Phase } from '../src/input.ts'

// createInput(host) binds keydown/keyup on the host EventTarget. Node has a
// global EventTarget, so we use a real one and feed it plain event-shaped
// objects (onKey only reads .key / .type / .repeat / .preventDefault).
// `document` and `navigator` are undefined under environment:'node', so the
// text-input-focus and gamepad guards short-circuit safely.

// A real (cancelable) Event carries the read-only `type` + working
// preventDefault/defaultPrevented; we only graft on the KeyboardEvent fields
// the input bus reads (`key`, `repeat`), which don't exist on Event.
function keyEvent(type: 'keydown' | 'keyup', key: string, repeat = false): Event {
  const ev = new Event(type, { cancelable: true }) as Event & { key: string; repeat: boolean }
  ev.key = key
  ev.repeat = repeat
  return ev
}

function press(host: EventTarget, key: string, repeat = false): Event {
  const ev = keyEvent('keydown', key, repeat)
  host.dispatchEvent(ev)
  return ev
}
function release(host: EventTarget, key: string): Event {
  const ev = keyEvent('keyup', key)
  host.dispatchEvent(ev)
  return ev
}

afterEach(() => { vi.unstubAllGlobals() })

describe('input: action edges', () => {
  it('fires start on rising edge and end on falling edge', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    const phases: Phase[] = []
    bus.on('jump', (p) => phases.push(p))
    bus.bind([' '], 'jump')
    press(host, ' ')
    expect(phases).toEqual(['start'])
    release(host, ' ')
    expect(phases).toEqual(['start', 'end'])
  })

  it('does not re-fire start while a non-repeat binding is held', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    const starts: number[] = []
    bus.on('fire', () => starts.push(1))
    bus.bind(['x'], 'fire', { repeat: false })
    press(host, 'x')
    // Re-pressing the same key (e.g. duplicate keydown) must not re-fire.
    press(host, 'x')
    expect(starts).toHaveLength(1)
  })

  it('re-fires start every evaluation while a repeat binding stays down', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let count = 0
    bus.on('move', () => count++)
    bus.bind(['d'], 'move', { repeat: true })
    press(host, 'd') // active goes true → 1
    // Another key event that re-evaluates while still held → repeat fires again.
    press(host, 'd')
    expect(count).toBe(2)
  })

  it('ignores OS auto-repeat keydown events (ke.repeat)', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let count = 0
    bus.on('jump', (p) => { if (p === 'start') count++ })
    bus.bind([' '], 'jump')
    press(host, ' ', true) // repeat flag set → early-returned, no state change
    expect(count).toBe(0)
  })
})

describe('input: key aliases + normalization', () => {
  it('maps Space alias to the space character', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    const phases: Phase[] = []
    bus.on('jump', (p) => phases.push(p))
    bus.bind(['Space'], 'jump') // alias → ' '
    press(host, ' ')
    expect(phases).toEqual(['start'])
  })

  it('maps Up alias to ArrowUp and lowercases single chars', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let up = false
    let leftFromW = false
    bus.on('north', (p) => { if (p === 'start') up = true })
    bus.on('left', (p) => { if (p === 'start') leftFromW = true })
    bus.bind(['Up'], 'north')
    bus.bind(['A'], 'left') // uppercase A normalizes to 'a'
    press(host, 'ArrowUp')
    press(host, 'a')
    expect(up).toBe(true)
    expect(leftFromW).toBe(true)
  })

  it('calls preventDefault for arrow/space keys, not for letters', () => {
    const host = new EventTarget()
    createInput(host)
    const spaceEv = press(host, ' ')
    expect(spaceEv.defaultPrevented).toBe(true)
    const letterEv = press(host, 'a')
    expect(letterEv.defaultPrevented).toBe(false)
  })
})

describe('input: multi-key bindings + isDown', () => {
  it('treats any bound key as the action being down (OR semantics)', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    bus.bind(['a', 'ArrowLeft'], 'left')
    expect(bus.isDown('left')).toBe(false)
    press(host, 'ArrowLeft')
    expect(bus.isDown('left')).toBe(true)
    // Press the other bound key too, then release the first — still down.
    press(host, 'a')
    release(host, 'ArrowLeft')
    expect(bus.isDown('left')).toBe(true)
    release(host, 'a')
    expect(bus.isDown('left')).toBe(false)
  })

  it('isDown returns false for an unknown action', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    expect(bus.isDown('nope')).toBe(false)
  })
})

describe('input: subscription + binding lifecycle', () => {
  it('unbind() stops the action from responding to its keys', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let count = 0
    bus.on('jump', (p) => { if (p === 'start') count++ })
    const unbind = bus.bind([' '], 'jump')
    press(host, ' ')
    expect(count).toBe(1)
    release(host, ' ')
    unbind()
    press(host, ' ')
    expect(count).toBe(1) // binding gone → no further fires
  })

  it('on() returns an unsubscribe that removes only that handler', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let a = 0
    let b = 0
    const offA = bus.on('jump', () => a++)
    bus.on('jump', () => b++)
    bus.bind([' '], 'jump')
    press(host, ' ')
    expect([a, b]).toEqual([1, 1])
    offA()
    release(host, ' ')
    press(host, ' ')
    expect(a).toBe(1) // unsubscribed
    expect(b).toBeGreaterThan(1)
  })

  it('fire() manually dispatches a start with detail', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let seen: unknown
    let phase: Phase | undefined
    bus.on('pause', (p, d) => { phase = p; seen = d })
    bus.fire('pause', { source: 'menu' })
    expect(phase).toBe('start')
    expect(seen).toEqual({ source: 'menu' })
  })

  it('destroy() detaches listeners and clears state', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    let count = 0
    bus.on('jump', () => count++)
    bus.bind([' '], 'jump')
    bus.destroy()
    press(host, ' ')
    expect(count).toBe(0)
    expect(bus.isDown('jump')).toBe(false)
  })
})

describe('input: gamepad polling', () => {
  it('maps a pressed gamepad button to a held action via pollGamepads', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    bus.bind(['GamepadButton0'], 'jump')
    const buttons = [{ pressed: true }, { pressed: false }]
    vi.stubGlobal('navigator', {
      getGamepads: () => [{ buttons }],
    })
    const phases: Phase[] = []
    bus.on('jump', (p) => phases.push(p))
    bus.pollGamepads()
    expect(phases).toEqual(['start'])
    expect(bus.isDown('jump')).toBe(true)
    // Release the button → next poll emits end.
    buttons[0]!.pressed = false
    bus.pollGamepads()
    expect(phases).toEqual(['start', 'end'])
  })

  it('no-ops cleanly when navigator/getGamepads is unavailable', () => {
    const host = new EventTarget()
    const bus = createInput(host)
    bus.bind(['GamepadButton0'], 'jump')
    expect(() => bus.pollGamepads()).not.toThrow()
    expect(bus.isDown('jump')).toBe(false)
  })
})
