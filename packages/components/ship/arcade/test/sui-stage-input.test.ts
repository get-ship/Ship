import { afterEach, describe, expect, it, vi } from 'vitest'

import { makeFakeCreateStage, lastFakeStage, resetFakeStage } from './fake-stage.ts'
vi.mock('@ship/arcade-engine', async () => {
  const actual = await vi.importActual<typeof import('@ship/arcade-engine')>('@ship/arcade-engine')
  return { ...actual, createStage: makeFakeCreateStage() }
})

import { mountStage, registerChild } from './harness.ts'

afterEach(() => resetFakeStage())

function keydown(key: string) {
  window.dispatchEvent(new KeyboardEvent('keydown', { key, bubbles: true }))
}
function keyup(key: string) {
  window.dispatchEvent(new KeyboardEvent('keyup', { key, bubbles: true }))
}

describe('<sui-stage-input> — maps keys to a logical action on the stage input bus', () => {
  it('registers as a custom element', () => {
    registerChild('sui-stage-input')
    expect(customElements.get('sui-stage-input')).toBeDefined()
  })

  it('a dispatched keydown for a mapped key fires the action (start phase)', async () => {
    await mountStage(`<sui-stage-input map="ArrowLeft" action="moveLeft"></sui-stage-input>`)
    const { stage } = lastFakeStage()
    const phases: string[] = []
    stage.input.on('moveLeft', (p) => phases.push(p))

    keydown('ArrowLeft')
    expect(phases).toContain('start')
    expect(stage.input.isDown('moveLeft')).toBe(true)

    keyup('ArrowLeft')
    expect(phases).toContain('end')
    expect(stage.input.isDown('moveLeft')).toBe(false)
  })

  it('honors the Space convenience alias (maps to the " " key)', async () => {
    await mountStage(`<sui-stage-input map="Space" action="fire"></sui-stage-input>`)
    const { stage } = lastFakeStage()
    let started = false
    stage.input.on('fire', (p) => { if (p === 'start') started = true })
    keydown(' ')
    expect(started).toBe(true)
  })

  it('an unmapped key does NOT fire the action', async () => {
    await mountStage(`<sui-stage-input map="ArrowUp" action="jump"></sui-stage-input>`)
    const { stage } = lastFakeStage()
    let fired = false
    stage.input.on('jump', () => { fired = true })
    keydown('ArrowDown')
    expect(fired).toBe(false)
  })

  it('maps multiple comma-separated keys to one action', async () => {
    await mountStage(`<sui-stage-input map="A,ArrowLeft" action="left"></sui-stage-input>`)
    const { stage } = lastFakeStage()
    let starts = 0
    stage.input.on('left', (p) => { if (p === 'start') starts++ })
    // letter keys normalize case-insensitively.
    keydown('a')
    keyup('a')
    keydown('ArrowLeft')
    expect(starts).toBeGreaterThanOrEqual(2)
  })
})
