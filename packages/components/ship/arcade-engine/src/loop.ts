// Fixed-timestep simulation + interpolated render loop.
//
// Sim ticks at a fixed rate (default 60 Hz) regardless of render fps. Render
// receives an `interp` value in [0, 1] so callers can interpolate visual state
// between sim ticks. Accumulator is clamped to avoid spiral-of-death when a
// tab gets backgrounded.

export interface LoopOptions {
  tickRate: number
  renderFps: number
  onTick: (dt: number, tickIndex: number) => void
  onRender: (interp: number, fps: number) => void
}

export interface Loop {
  start(): void
  stop(): void
  pause(): void
  resume(): void
  readonly paused: boolean
}

export function createLoop(opts: LoopOptions): Loop {
  const tickDt = 1 / Math.max(1, opts.tickRate)
  const maxAccum = tickDt * 5
  let raf = 0
  let lastTime = 0
  let accum = 0
  let tickIndex = 0
  let paused = false
  let stopped = true

  let fpsSamples = 0
  let fpsElapsed = 0
  let fps = 0

  function frame(now: number) {
    raf = requestAnimationFrame(frame)
    if (paused) {
      lastTime = now
      return
    }
    const dtMs = now - lastTime
    lastTime = now
    const dt = dtMs / 1000

    fpsSamples += 1
    fpsElapsed += dt
    if (fpsElapsed >= 0.5) {
      fps = fpsSamples / fpsElapsed
      fpsSamples = 0
      fpsElapsed = 0
    }

    accum += dt
    if (accum > maxAccum) accum = maxAccum
    while (accum >= tickDt) {
      opts.onTick(tickDt, tickIndex++)
      accum -= tickDt
    }
    opts.onRender(accum / tickDt, fps)
  }

  return {
    start() {
      if (!stopped) return
      stopped = false
      paused = false
      lastTime = performance.now()
      accum = 0
      raf = requestAnimationFrame(frame)
    },
    stop() {
      if (stopped) return
      stopped = true
      cancelAnimationFrame(raf)
    },
    pause() { paused = true },
    resume() {
      paused = false
      lastTime = performance.now()
    },
    get paused() { return paused },
  }
}
