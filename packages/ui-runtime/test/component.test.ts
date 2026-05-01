import { describe, expect, it, vi } from 'vitest'
import { defineComponent, camelOf } from '../src/component'
import { signal } from '../src/signal'
import type { Signal } from '../src/signal'

let tagCounter = 0
const nextTag = () => `ship-set-test-${++tagCounter}`

describe('camelOf', () => {
  it('single word', () => expect(camelOf('count')).toBe('count'))
  it('kebab two', () => expect(camelOf('fetch-url')).toBe('fetchUrl'))
  it('kebab three', () => expect(camelOf('a-b-c')).toBe('aBC'))
})

describe('defineComponent', () => {
  it('registers a custom element', () => {
    const tag = nextTag()
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    expect(customElements.get(tag)).toBeDefined()
  })

  it('is idempotent', () => {
    const tag = nextTag()
    const f = vi.fn(() => () => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, f)
    defineComponent(tag, { observedAttrs: [], props: {} }, f)
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(f).toHaveBeenCalledTimes(1)
    el.remove()
  })

  it('runs factory on connect, cleanup on disconnect', () => {
    const tag = nextTag()
    const cleanup = vi.fn()
    const factory = vi.fn(() => cleanup)
    defineComponent(tag, { observedAttrs: [], props: {} }, factory)
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(factory).toHaveBeenCalledTimes(1)
    expect(cleanup).not.toHaveBeenCalled()
    el.remove()
    expect(cleanup).toHaveBeenCalledTimes(1)
  })

  it('sets data-component for CSS scoping', () => {
    const tag = nextTag()
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {})
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(el.dataset.component).toBe(tag)
    el.remove()
  })

  it('routes attribute changes through prop signals', () => {
    const tag = nextTag()
    let captured: Signal<unknown> | undefined
    defineComponent(
      tag,
      {
        observedAttrs: ['count'],
        props: { count: { camel: 'count', decode: (r) => Number(r) } },
      },
      (_host, propSignals) => {
        propSignals.count = signal(0)
        captured = propSignals.count
        return () => {}
      },
    )
    const el = document.createElement(tag)
    document.body.appendChild(el)
    el.setAttribute('count', '7')
    expect(captured!()).toBe(7)
    el.remove()
  })

  it('cleanup error does not throw', () => {
    const tag = nextTag()
    const spy = vi.spyOn(console, 'error').mockImplementation(() => {})
    defineComponent(tag, { observedAttrs: [], props: {} }, () => () => {
      throw new Error('boom')
    })
    const el = document.createElement(tag)
    document.body.appendChild(el)
    expect(() => el.remove()).not.toThrow()
    expect(spy).toHaveBeenCalled()
    spy.mockRestore()
  })
})
