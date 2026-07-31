import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import '../src/sui-log-stream/sui-log-stream.compiled.js'
import { createContainerRegistry, flush, mount } from '@ship/test-setup'

const reg = createContainerRegistry()

// JSDOM has no EventSource — stub one we can drive from the tests.
class FakeEventSource {
  static instances: FakeEventSource[] = []
  url: string
  withCredentials: boolean
  onopen: ((this: FakeEventSource, ev: Event) => unknown) | null = null
  onerror: ((this: FakeEventSource, ev: Event) => unknown) | null = null
  listeners = new Map<string, Array<(ev: MessageEvent) => void>>()
  closed = false
  constructor(url: string, init?: { withCredentials?: boolean }) {
    this.url = url
    this.withCredentials = !!init?.withCredentials
    FakeEventSource.instances.push(this)
  }
  addEventListener(type: string, fn: (ev: MessageEvent) => void) {
    const arr = this.listeners.get(type) ?? []
    arr.push(fn)
    this.listeners.set(type, arr)
  }
  emit(type: string, data: string) {
    for (const fn of this.listeners.get(type) ?? []) {
      fn({ data } as MessageEvent)
    }
  }
  close() {
    this.closed = true
  }
}

beforeEach(() => {
  FakeEventSource.instances = []
  ;(globalThis as Record<string, unknown>).EventSource = FakeEventSource
})

afterEach(() => reg.cleanup())

const shadow = (el: HTMLElement) => el.shadowRoot!

describe('<sui-log-stream>', () => {

  it('renders a <pre part="scroll"> in shadow DOM', async () => {
    const { el } = mount(reg, 'sui-log-stream')
    await flush()
    expect(shadow(el).querySelector('pre[part="scroll"]')).not.toBeNull()
  })

  it('shows the empty placeholder by default', async () => {
    const { el } = mount(reg, 'sui-log-stream')
    await flush()
    const pre = shadow(el).querySelector('pre')!
    expect(pre.textContent).toBe('(no output yet)')
    expect(pre.hasAttribute('data-empty')).toBe(true)
  })

  it('does NOT open a connection when active is false', async () => {
    mount(reg, 'sui-log-stream', { url: '/api/logs/stream' })
    await flush()
    expect(FakeEventSource.instances.length).toBe(0)
  })

  it('opens an EventSource when active and url are set', async () => {
    mount(reg, 'sui-log-stream', { url: '/api/logs/stream', active: true })
    await flush()
    expect(FakeEventSource.instances.length).toBe(1)
    expect(FakeEventSource.instances[0].url).toBe('/api/logs/stream')
  })

  it('emits sui-stream-open on EventSource open', async () => {
    const { el } = mount(reg, 'sui-log-stream', {
      url: '/api/logs/stream',
      active: true,
    })
    await flush()
    const opens = vi.fn()
    el.addEventListener('sail-stream-open', opens)
    FakeEventSource.instances[0].onopen?.(new Event('open'))
    expect(opens).toHaveBeenCalledTimes(1)
  })

  it('appends payloads to the pre when SSE messages arrive', async () => {
    const { el } = mount(reg, 'sui-log-stream', {
      url: '/api/logs/stream',
      active: true,
    })
    await flush()
    FakeEventSource.instances[0].emit('log', 'first line\n')
    FakeEventSource.instances[0].emit('log', 'second line\n')
    await flush()
    const pre = shadow(el).querySelector('pre')!
    expect(pre.textContent).toBe('first line\nsecond line\n')
    expect(pre.hasAttribute('data-empty')).toBe(false)
  })

  it('fires sui-log-tick with byte count for each chunk', async () => {
    const { el } = mount(reg, 'sui-log-stream', {
      url: '/api/logs/stream',
      active: true,
    })
    await flush()
    const ticks: number[] = []
    el.addEventListener('sail-log-tick', (e) => {
      const ev = e as CustomEvent<{ bytes: number }>
      ticks.push(ev.detail.bytes)
    })
    FakeEventSource.instances[0].emit('log', 'abc')
    FakeEventSource.instances[0].emit('log', 'hello world')
    expect(ticks).toEqual([3, 11])
  })

  it('emits sui-stream-error when the connection errors', async () => {
    const { el } = mount(reg, 'sui-log-stream', {
      url: '/api/logs/stream',
      active: true,
    })
    await flush()
    const errs = vi.fn()
    el.addEventListener('sail-stream-error', errs)
    FakeEventSource.instances[0].onerror?.(new Event('error'))
    expect(errs).toHaveBeenCalledTimes(1)
  })

  it('closes the connection and emits sui-stream-close when active flips off', async () => {
    const { el } = mount(reg, 'sui-log-stream', {
      url: '/api/logs/stream',
      active: true,
    })
    await flush()
    const closes = vi.fn()
    el.addEventListener('sail-stream-close', closes)
    el.removeAttribute('active')
    await flush()
    expect(FakeEventSource.instances[0].closed).toBe(true)
    expect(closes).toHaveBeenCalledTimes(1)
  })

  it('clear() resets the buffer back to the empty placeholder', async () => {
    const { el } = mount(reg, 'sui-log-stream', {
      url: '/api/logs/stream',
      active: true,
    }) as { el: HTMLElement & { clear: () => void } }
    await flush()
    FakeEventSource.instances[0].emit('log', 'hello')
    await flush()
    expect(shadow(el).querySelector('pre')!.textContent).toBe('hello')
    el.clear()
    expect(shadow(el).querySelector('pre')!.textContent).toBe('(no output yet)')
    expect(shadow(el).querySelector('pre')!.hasAttribute('data-empty')).toBe(true)
  })
})
