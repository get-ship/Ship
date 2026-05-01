import { describe, expect, it } from 'vitest'
import { compileAttrs } from '../src/bindings/attr-compile'

const attr = (markup: string) => {
  const doc = new DOMParser().parseFromString(
    `<!doctype html><html><body>${markup}</body></html>`,
    'text/html',
  )
  return doc.body.firstElementChild!
}

describe('compileAttrs', () => {
  it('classifies static attributes', () => {
    const el = attr('<a href="/about">x</a>')
    const attrs = compileAttrs(el, '<a>')
    expect(attrs).toHaveLength(1)
    expect(attrs[0]!.kind).toBe('static')
  })

  it('classifies expression attribute bindings', () => {
    const el = attr('<a href={url}>x</a>')
    const attrs = compileAttrs(el, '<a>')
    expect(attrs[0]!.kind).toBe('attr')
  })

  it('classifies bind:value, bind:checked', () => {
    const el = attr('<input bind:value={query} bind:checked={agreed} />')
    const kinds = compileAttrs(el, '<input>').map((a) => a.kind)
    expect(kinds).toContain('bind-value')
    expect(kinds).toContain('bind-checked')
  })

  it('parses on:event with modifiers', () => {
    const el = attr('<button on:click|preventDefault|stop={fn}>x</button>')
    const attrs = compileAttrs(el, '<button>')
    expect(attrs).toHaveLength(1)
    const a = attrs[0]!
    if (a.kind !== 'on') throw new Error('expected on')
    expect(a.event).toBe('click')
    expect(a.modifiers.map((m) => m.kind)).toEqual([
      'preventDefault',
      'stopPropagation',
    ])
  })

  it('parses key modifier with arg', () => {
    const el = attr('<input on:keydown|key:Enter={fn} />')
    const attrs = compileAttrs(el, '<input>')
    const a = attrs[0]!
    if (a.kind !== 'on') throw new Error('expected on')
    // HTML attribute names are lowercased by the parser; the
    // runtime's matchesKey is case-insensitive on the key name so
    // the lowercased arg matches the corresponding event.key.
    expect(a.modifiers[0]).toEqual({ kind: 'key', arg: 'enter' })
  })

  it('parses class:* and style:* bindings', () => {
    const el = attr('<div class:active={isOn} style:--gap={spacing}>x</div>')
    const kinds = compileAttrs(el, '<div>').map((a) => a.kind)
    expect(kinds).toEqual(['class', 'style'])
  })

  it('rejects on:event without {expr}', () => {
    const el = attr('<button on:click="literal">x</button>')
    expect(() => compileAttrs(el, '<button>')).toThrow(/handler must be/)
  })
})
