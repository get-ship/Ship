import { describe, expect, it } from 'vitest'

// children.ts is a pure DOM->spec parser (no engine, no Pixi); test it directly.
import { readChildSpecs, parseEmitterSize } from '../src/sui-stage/children.ts'

/** Build a detached host with the given light-DOM markup. readChildSpecs only
 *  reads `:scope > <tag>` children + their attributes, so a plain div host
 *  with arcade child tags is a faithful stand-in for <sui-stage>. */
function host(markup: string): HTMLElement {
  const el = document.createElement('div')
  el.innerHTML = markup
  return el
}

describe('readChildSpecs — light-DOM parsing for the stage', () => {
  it('parses sui-input: csv keys, action, repeat default true', () => {
    const specs = readChildSpecs(host(`<sui-input map="A, ArrowLeft ,  " action="left"></sui-input>`))
    expect(specs.inputs).toHaveLength(1)
    expect(specs.inputs[0]!.keys).toEqual(['A', 'ArrowLeft']) // trimmed, empties dropped
    expect(specs.inputs[0]!.action).toBe('left')
    expect(specs.inputs[0]!.repeat).toBe(true)
  })

  it('parses sui-input repeat="false" as false', () => {
    const specs = readChildSpecs(host(`<sui-input map="X" action="x" repeat="false"></sui-input>`))
    expect(specs.inputs[0]!.repeat).toBe(false)
  })

  it('parses sui-sfx: pitch-range "a..b" + numeric volume + bus', () => {
    const specs = readChildSpecs(host(`<sui-sfx src="/s.wav" on="hit" volume="0.7" pitch-range="0.9..1.3" bus="music"></sui-sfx>`))
    expect(specs.sfx[0]!).toMatchObject({
      src: '/s.wav', action: 'hit', volume: 0.7, bus: 'music', pitchRange: [0.9, 1.3],
    })
  })

  it('parses sui-emitter: numeric fields + on action + size string passthrough', () => {
    const specs = readChildSpecs(host(`<sui-emitter id="e" kind="cone" x="3" y="4" rate="5" burst="9" on="hit" size="6..1"></sui-emitter>`))
    const e = specs.emitters[0]!
    expect(e.id).toBe('e')
    expect(e.kind).toBe('cone')
    expect(e.x).toBe(3)
    expect(e.y).toBe(4)
    expect(e.rate).toBe(5)
    expect(e.burst).toBe(9)
    expect(e.action).toBe('hit')
    expect(parseEmitterSize(e.size)).toEqual([6, 1])
  })

  it('parses a sui-sprite entity: src, frames grid, anchor preset, numeric transform', () => {
    const specs = readChildSpecs(host(
      `<sui-sprite src="/h.png" tag="hero" x="10" y="20" vx="1" frames="3x2" frame="2" anchor="bottom-right"></sui-sprite>`,
    ))
    const e = specs.entities[0]!
    expect(e.visual).toBe('sprite')
    expect(e.src).toBe('/h.png')
    expect(e.x).toBe(10)
    expect(e.y).toBe(20)
    expect(e.vx).toBe(1)
    expect(e.frames).toEqual([3, 2])
    expect(e.frame).toBe(2)
    expect(e.anchor).toEqual([1, 1]) // bottom-right preset
  })

  it('falls back to anchor center for an unknown anchor token', () => {
    const e = readChildSpecs(host(`<sui-sprite src="/x.png" anchor="nonsense"></sui-sprite>`)).entities[0]!
    expect(e.anchor).toEqual([0.5, 0.5])
  })

  it('parses a numeric anchor pair "0.2,0.8"', () => {
    const e = readChildSpecs(host(`<sui-sprite src="/x.png" anchor="0.2,0.8"></sui-sprite>`)).entities[0]!
    expect(e.anchor).toEqual([0.2, 0.8])
  })

  it('drops a malformed frames string (undefined, not a bogus pair)', () => {
    const e = readChildSpecs(host(`<sui-sprite src="/x.png" frames="bad"></sui-sprite>`)).entities[0]!
    expect(e.frames).toBeUndefined()
  })

  it('parses a sui-shape entity with shape kind + dims + polygon points', () => {
    const e = readChildSpecs(host(
      `<sui-shape kind="polygon" points="0,0 4 8, 2" width="12" height="6" fill="accent"></sui-shape>`,
    )).entities[0]!
    expect(e.visual).toBe('shape')
    expect(e.shape).toBe('polygon')
    expect(e.width).toBe(12)
    expect(e.height).toBe(6)
    expect(e.points).toEqual([0, 0, 4, 8, 2]) // split on whitespace+commas, finite only
    expect(e.fill).toBe('accent')
  })

  it('reads a nested sui-collider into the parent entity spec', () => {
    const e = readChildSpecs(host(`
      <sui-shape kind="rect" width="10" height="10">
        <sui-collider kind="circle" radius="6" offset-x="2" offset-y="-1" group="ball" collides-with="wall, goal" trigger></sui-collider>
      </sui-shape>
    `)).entities[0]!
    expect(e.collider).not.toBeNull()
    expect(e.collider!).toMatchObject({
      kind: 'circle', radius: 6, offsetX: 2, offsetY: -1, group: 'ball',
      collidesWith: ['wall', 'goal'], trigger: true,
    })
  })

  it('leaves collider null when no sui-collider child is present', () => {
    const e = readChildSpecs(host(`<sui-shape kind="rect"></sui-shape>`)).entities[0]!
    expect(e.collider).toBeNull()
  })

  it('only reads DIRECT children (a nested sui-input is ignored)', () => {
    // querySelectorAll(':scope > sui-input') must not match a grandchild input.
    const specs = readChildSpecs(host(`
      <sui-input map="A" action="top"></sui-input>
      <sui-shape kind="rect"><sui-input map="B" action="buried"></sui-input></sui-shape>
    `))
    expect(specs.inputs.map((i) => i.action)).toEqual(['top'])
  })

  it('defaults numeric fields when the attribute is absent or empty', () => {
    const e = readChildSpecs(host(`<sui-shape kind="rect" x=""></sui-shape>`)).entities[0]!
    expect(e.x).toBe(0)
    expect(e.width).toBe(10) // shape default width
    expect(e.scale).toBe(1)
    expect(e.alpha).toBe(1)
  })
})

describe('parseEmitterSize', () => {
  it('parses "start..end"', () => {
    expect(parseEmitterSize('8..2')).toEqual([8, 2])
  })
  it('defaults end to 0 when only a start is given', () => {
    expect(parseEmitterSize('5')).toEqual([5, 0])
  })
  it('defaults to [3, 0] on garbage', () => {
    expect(parseEmitterSize('abc')).toEqual([3, 0])
  })
})
