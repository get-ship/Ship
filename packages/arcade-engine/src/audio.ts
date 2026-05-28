// Web Audio: three buses (sfx, music, ui), per-bus volume, preload + cached
// AudioBuffers, one-shot playback with pitch range. Context is lazy because
// browsers require a user gesture before creating one — first play() forces it.

export type Bus = 'sfx' | 'music' | 'ui'

export interface PlayOptions {
  volume?: number
  pitch?: number
  bus?: Bus
}

export interface AudioGraph {
  preload(src: string): Promise<void>
  play(src: string, opts?: PlayOptions): void
  setBusVolume(bus: Bus, v: number): void
  suspend(): void
  resume(): void
  destroy(): void
}

export function createAudio(): AudioGraph {
  let ctx: AudioContext | null = null
  const buses: Record<Bus, GainNode | null> = { sfx: null, music: null, ui: null }
  const buffers = new Map<string, AudioBuffer>()
  const pending = new Map<string, Promise<AudioBuffer>>()
  const busVolumes: Record<Bus, number> = { sfx: 1, music: 1, ui: 1 }
  let destroyed = false

  function ensureCtx(): AudioContext | null {
    if (destroyed) return null
    if (ctx) return ctx
    const Ctor = (globalThis as unknown as { AudioContext?: typeof AudioContext; webkitAudioContext?: typeof AudioContext }).AudioContext
      ?? (globalThis as unknown as { webkitAudioContext?: typeof AudioContext }).webkitAudioContext
    if (!Ctor) return null
    ctx = new Ctor()
    for (const b of ['sfx', 'music', 'ui'] as const) {
      const g = ctx.createGain()
      g.gain.value = busVolumes[b]
      g.connect(ctx.destination)
      buses[b] = g
    }
    return ctx
  }

  async function loadBuffer(src: string): Promise<AudioBuffer> {
    const c = ensureCtx()
    if (!c) throw new Error('AudioContext unavailable')
    const existing = buffers.get(src)
    if (existing) return existing
    const inflight = pending.get(src)
    if (inflight) return inflight
    const p = (async () => {
      const res = await fetch(src)
      const data = await res.arrayBuffer()
      const buf = await c.decodeAudioData(data)
      buffers.set(src, buf)
      pending.delete(src)
      return buf
    })()
    pending.set(src, p)
    return p
  }

  return {
    async preload(src) { await loadBuffer(src) },
    play(src, opts) {
      const c = ensureCtx()
      if (!c) return
      const bus = opts?.bus ?? 'sfx'
      const target = buses[bus]
      if (!target) return
      const buf = buffers.get(src)
      if (!buf) {
        void loadBuffer(src).then(() => this.play(src, opts))
        return
      }
      const source = c.createBufferSource()
      source.buffer = buf
      source.playbackRate.value = opts?.pitch ?? 1
      const gain = c.createGain()
      gain.gain.value = opts?.volume ?? 1
      source.connect(gain)
      gain.connect(target)
      source.start()
    },
    setBusVolume(bus, v) {
      busVolumes[bus] = v
      const g = buses[bus]
      if (g) g.gain.value = v
    },
    suspend() { ctx?.suspend() },
    resume() { ctx?.resume() },
    destroy() {
      destroyed = true
      ctx?.close()
      ctx = null
      buffers.clear()
      pending.clear()
    },
  }
}
