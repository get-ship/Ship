import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

afterEach(() => resetFakeStage())

describe('<sui-hud> — overlay binds named stage signals to data-bind descendants', () => {
  it('registers as a custom element', () => {
    registerChild('sui-hud')
    expect(customElements.get('sui-hud')).toBeDefined()
  })

  it('renders the score: set("score", v) flows into a data-bind="score" node', async () => {
    const { host } = await mountStage(`
      <sui-hud align="top">
        <span data-bind="score"></span>
      </sui-hud>
    `)
    const scoreEl = host.querySelector('[data-bind="score"]')!
    // The stage pre-binds the `score` signal (initial 0).
    expect(scoreEl.textContent).toBe('0')

    host.set('score', 1234)
    expect(scoreEl.textContent).toBe('1234')

    host.set('score', 42)
    expect(scoreEl.textContent).toBe('42')
  })

  it('renders the state: state transitions flow into a data-bind="state" node', async () => {
    const { host } = await mountStage(`
      <sui-hud align="center">
        <span data-bind="state"></span>
      </sui-hud>
    `)
    const stateEl = host.querySelector('[data-bind="state"]')!
    expect(stateEl.textContent).toBe('idle')

    host.start() // stage -> playing -> onState writes the `state` signal
    expect(stateEl.textContent).toBe('playing')
    host.pause()
    expect(stateEl.textContent).toBe('paused')
  })

  it('reset() zeroes the bound score', async () => {
    const { host } = await mountStage(`
      <sui-hud><span data-bind="score"></span></sui-hud>
    `)
    const scoreEl = host.querySelector('[data-bind="score"]')!
    host.set('score', 99)
    expect(scoreEl.textContent).toBe('99')
    host.reset() // controller sets state.score = 0
    expect(scoreEl.textContent).toBe('0')
  })

  it('a director-registered signal binds via host.bind() + writes through host.set()', async () => {
    // The director can register arbitrary named signals; the HUD re-wires so a
    // data-bind for the new name follows it. Uses the runtime's own signal.
    const { host } = await mountStage(`
      <sui-hud><span data-bind="lives"></span></sui-hud>
    `)
    const runtime = await import('@ship/sail-runtime')
    const lives = runtime.signal(3)
    host.bind('lives', lives)

    const livesEl = host.querySelector('[data-bind="lives"]')!
    expect(livesEl.textContent).toBe('3')
    host.set('lives', 2)
    expect(livesEl.textContent).toBe('2')
  })

  it('writes data-bind-attr targets as attributes, not textContent', async () => {
    const { host } = await mountStage(`
      <sui-hud><progress data-bind="score" data-bind-attr="value"></progress></sui-hud>
    `)
    // Anchor on the binding contract (data-bind / data-bind-attr), not a
    // styling class.
    const bar = host.querySelector('[data-bind-attr="value"]')!
    host.set('score', 7)
    expect(bar.getAttribute('value')).toBe('7')
    expect(bar.textContent).toBe('')
  })
})
