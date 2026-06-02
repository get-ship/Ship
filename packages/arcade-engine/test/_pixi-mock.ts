// Lightweight stand-in for the vendored PixiJS bundle.
//
// world.ts / particles.ts / stage.ts `import * as PIXI from
// '../vendor/pixi.js/pixi.mjs'` — the real bundle is ~800 KB and wants a live
// WebGL/canvas renderer. None of the engine LOGIC depends on Pixi actually
// drawing: it only depends on the display objects recording the transforms /
// child list mutations the engine pushes into them. This mock records those so
// tests can assert on them, while staying fast and DOM-free.
//
// Tests wire it in with:
//   vi.mock('../vendor/pixi.js/pixi.mjs', () => pixiMockModule())
// and can reach the recorded state through the exported `pixiMockState`.

export interface MockTransform {
  x: number
  y: number
  rotation: number
  alpha: number
  zIndex: number
  visible: boolean
  scaleX: number
  tint?: number
  destroyed: boolean
  /** Painting ops recorded in call order: e.g. 'clear', 'circle', 'fill'. */
  ops: string[]
  children: MockDisplay[]
}

export interface MockDisplay extends MockTransform {
  scale: { set(v: number): void }
  anchor: { set(x: number, y: number): void }
  texture: unknown
  clear(): MockDisplay
  rect(...a: number[]): MockDisplay
  circle(...a: number[]): MockDisplay
  poly(p: number[]): MockDisplay
  fill(c?: unknown): MockDisplay
  stroke(o?: unknown): MockDisplay
  moveTo(x: number, y: number): MockDisplay
  lineTo(x: number, y: number): MockDisplay
  addChild(c: MockDisplay): MockDisplay
  removeChild(c: MockDisplay): MockDisplay
  removeChildren(): MockDisplay[]
  destroy(opts?: unknown): void
}

function makeDisplay(): MockDisplay {
  const d: MockDisplay = {
    x: 0,
    y: 0,
    rotation: 0,
    alpha: 1,
    zIndex: 0,
    visible: true,
    scaleX: 1,
    destroyed: false,
    ops: [],
    children: [],
    scale: { set(v: number) { d.scaleX = v } },
    anchor: { set() { /* recorded by ops if needed */ } },
    texture: null,
    clear() { d.ops.push('clear'); return d },
    rect() { d.ops.push('rect'); return d },
    circle() { d.ops.push('circle'); return d },
    poly() { d.ops.push('poly'); return d },
    fill() { d.ops.push('fill'); return d },
    stroke() { d.ops.push('stroke'); return d },
    moveTo() { d.ops.push('moveTo'); return d },
    lineTo() { d.ops.push('lineTo'); return d },
    addChild(c) { d.children.push(c); return c },
    removeChild(c) {
      const i = d.children.indexOf(c)
      if (i >= 0) d.children.splice(i, 1)
      return c
    },
    removeChildren() {
      const old = d.children.slice()
      d.children.length = 0
      return old
    },
    destroy() { d.destroyed = true },
  }
  return d
}

export interface PixiMockState {
  apps: MockApp[]
  assetsLoad: (src: string) => Promise<unknown>
  reset(): void
}

export interface MockApp {
  inited: boolean
  initOpts: unknown
  destroyed: boolean
  renderCount: number
  canvas: { setAttribute(k: string, v: string): void; style: { cssText: string } }
  stage: MockDisplay & { sortableChildren: boolean }
  init(opts: unknown): Promise<void>
  render(): void
  destroy(...a: unknown[]): void
}

export const pixiMockState: PixiMockState = {
  apps: [],
  assetsLoad: (src: string) => Promise.resolve({ src, width: 64, height: 64, source: {} }),
  reset() {
    this.apps = []
    this.assetsLoad = (src: string) => Promise.resolve({ src, width: 64, height: 64, source: {} })
  },
}

/** Factory passed to `vi.mock(path, () => pixiMockModule())`. */
export function pixiMockModule() {
  class Graphics {
    constructor() { return makeDisplay() }
  }
  class Container {
    constructor() { return makeDisplay() }
  }
  class Sprite {
    constructor() {
      const d = makeDisplay()
      return d
    }
  }
  class Application {
    inited = false
    initOpts: unknown = null
    destroyed = false
    renderCount = 0
    canvas = { setAttribute() {}, style: { cssText: '' } }
    stage = Object.assign(makeDisplay(), { sortableChildren: false })
    constructor() {
      pixiMockState.apps.push(this as unknown as MockApp)
    }
    async init(opts: unknown) {
      this.inited = true
      this.initOpts = opts
    }
    render() { this.renderCount++ }
    destroy() { this.destroyed = true }
  }
  const Assets = { load: (src: string) => pixiMockState.assetsLoad(src) }
  class Texture {
    constructor(public opts: unknown) {}
  }
  class Rectangle {
    constructor(public x: number, public y: number, public w: number, public h: number) {}
  }
  const extensions = { remove() {} }
  const EventSystem = {}
  return { Graphics, Container, Sprite, Application, Assets, Texture, Rectangle, extensions, EventSystem }
}
