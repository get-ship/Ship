import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { createAudio } from '../src/audio.ts'

// audio.ts builds a Web Audio graph lazily off globalThis.AudioContext, and
// fetches+decodes buffers. We install a fully-mocked AudioContext (recording
// gain/source/connect calls) and a fake fetch+decodeAudioData so the state
// machine — lazy ctx creation, bus routing, volume, suspend/resume/destroy —
// is observable without a real audio device.

interface MockGain {
  gain: { value: number }
  connections: unknown[]
  connect(t: unknown): void
}
interface MockSource {
  buffer: unknown
  playbackRate: { value: number }
  started: boolean
  connect(t: unknown): void
  start(): void
}

class MockAudioContext {
  static instances: MockAudioContext[] = []
  destination = { id: 'destination' }
  state: 'running' | 'suspended' | 'closed' = 'running'
  gains: MockGain[] = []
  sources: MockSource[] = []
  decodeCalls = 0
  closed = false
  constructor() { MockAudioContext.instances.push(this) }
  createGain(): MockGain {
    const g: MockGain = { gain: { value: 1 }, connections: [], connect(t) { this.connections.push(t) } }
    this.gains.push(g)
    return g
  }
  createBufferSource(): MockSource {
    const s: MockSource = {
      buffer: null,
      playbackRate: { value: 1 },
      started: false,
      connect() {},
      start() { this.started = true },
    }
    this.sources.push(s)
    return s
  }
  decodeAudioData(_data: ArrayBuffer): Promise<{ tag: 'buffer' }> {
    this.decodeCalls++
    return Promise.resolve({ tag: 'buffer' })
  }
  suspend() { this.state = 'suspended'; return Promise.resolve() }
  resume() { this.state = 'running'; return Promise.resolve() }
  close() { this.closed = true; this.state = 'closed'; return Promise.resolve() }
}

function installAudio() {
  MockAudioContext.instances = []
  vi.stubGlobal('AudioContext', MockAudioContext)
  vi.stubGlobal('fetch', vi.fn(async () => ({ arrayBuffer: async () => new ArrayBuffer(8) })))
}

beforeEach(() => { installAudio() })
afterEach(() => { vi.unstubAllGlobals(); vi.restoreAllMocks() })

describe('audio: lazy context creation', () => {
  it('does not create an AudioContext until the first play/preload', () => {
    createAudio()
    expect(MockAudioContext.instances).toHaveLength(0)
  })

  it('creates exactly one context with three bus gains on first preload', async () => {
    const audio = createAudio()
    await audio.preload('boom.wav')
    expect(MockAudioContext.instances).toHaveLength(1)
    // sfx + music + ui gain nodes, each wired to destination.
    const ctx = MockAudioContext.instances[0]!
    expect(ctx.gains.filter((g) => g.connections.includes(ctx.destination))).toHaveLength(3)
  })

  it('reuses the same context across calls', async () => {
    const audio = createAudio()
    await audio.preload('a.wav')
    await audio.preload('b.wav')
    expect(MockAudioContext.instances).toHaveLength(1)
  })
})

describe('audio: buffer caching', () => {
  it('decodes each distinct src exactly once', async () => {
    const audio = createAudio()
    await audio.preload('a.wav')
    await audio.preload('a.wav') // cached → no second decode
    const ctx = MockAudioContext.instances[0]!
    expect(ctx.decodeCalls).toBe(1)
    expect((globalThis.fetch as ReturnType<typeof vi.fn>).mock.calls).toHaveLength(1)
  })

  it('dedupes concurrent in-flight loads of the same src', async () => {
    const audio = createAudio()
    await Promise.all([audio.preload('x.wav'), audio.preload('x.wav')])
    const ctx = MockAudioContext.instances[0]!
    expect(ctx.decodeCalls).toBe(1)
  })
})

describe('audio: playback routing', () => {
  it('routes a preloaded sound through the requested bus with pitch + volume', async () => {
    const audio = createAudio()
    await audio.preload('shot.wav')
    const ctx = MockAudioContext.instances[0]!
    audio.play('shot.wav', { bus: 'music', pitch: 1.5, volume: 0.3 })
    expect(ctx.sources).toHaveLength(1)
    const src = ctx.sources[0]!
    expect(src.started).toBe(true)
    expect(src.buffer).toEqual({ tag: 'buffer' })
    expect(src.playbackRate.value).toBe(1.5)
    // A per-shot gain carries the volume.
    const shotGain = ctx.gains.find((g) => g.gain.value === 0.3)
    expect(shotGain).toBeDefined()
  })

  it('defaults to the sfx bus and unit pitch/volume', async () => {
    const audio = createAudio()
    await audio.preload('s.wav')
    const ctx = MockAudioContext.instances[0]!
    audio.play('s.wav')
    const src = ctx.sources[0]!
    expect(src.playbackRate.value).toBe(1)
    expect(src.started).toBe(true)
  })

  it('lazily loads then plays a sound that was not preloaded', async () => {
    const audio = createAudio()
    audio.play('late.wav') // not preloaded → kicks off a load
    // Drain the loadBuffer().then(play) microtasks.
    await new Promise((r) => setTimeout(r, 0))
    const ctx = MockAudioContext.instances[0]!
    expect(ctx.sources.length).toBeGreaterThanOrEqual(1)
    expect(ctx.sources[0]!.started).toBe(true)
  })
})

describe('audio: bus volume', () => {
  it('setBusVolume before ctx creation is applied when the gain is built', async () => {
    const audio = createAudio()
    audio.setBusVolume('music', 0.2) // no ctx yet — stored
    await audio.preload('m.wav') // ctx built; music gain should read 0.2
    const ctx = MockAudioContext.instances[0]!
    // music is the 2nd-created gain (sfx, music, ui order).
    expect(ctx.gains[1]!.gain.value).toBe(0.2)
  })

  it('setBusVolume after ctx creation updates the live gain', async () => {
    const audio = createAudio()
    await audio.preload('m.wav')
    const ctx = MockAudioContext.instances[0]!
    audio.setBusVolume('sfx', 0.5)
    expect(ctx.gains[0]!.gain.value).toBe(0.5)
  })
})

describe('audio: lifecycle', () => {
  it('suspend/resume toggle the context state', async () => {
    const audio = createAudio()
    await audio.preload('a.wav')
    const ctx = MockAudioContext.instances[0]!
    audio.suspend()
    expect(ctx.state).toBe('suspended')
    audio.resume()
    expect(ctx.state).toBe('running')
  })

  it('destroy closes the context and blocks further context creation', async () => {
    const audio = createAudio()
    await audio.preload('a.wav')
    const ctx = MockAudioContext.instances[0]!
    audio.destroy()
    expect(ctx.closed).toBe(true)
    // After destroy, play() must not spin up a fresh context.
    audio.play('b.wav')
    expect(MockAudioContext.instances).toHaveLength(1)
  })
})

describe('audio: no Web Audio support', () => {
  it('play is a silent no-op when AudioContext is unavailable', () => {
    vi.stubGlobal('AudioContext', undefined)
    vi.stubGlobal('webkitAudioContext', undefined)
    const audio = createAudio()
    expect(() => audio.play('x.wav')).not.toThrow()
  })

  it('preload rejects when there is no AudioContext', async () => {
    vi.stubGlobal('AudioContext', undefined)
    vi.stubGlobal('webkitAudioContext', undefined)
    const audio = createAudio()
    await expect(audio.preload('x.wav')).rejects.toThrow('AudioContext unavailable')
  })
})
