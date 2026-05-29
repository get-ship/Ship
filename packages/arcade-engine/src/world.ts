// Entity registry + Pixi scene-graph adapter. Engine consumers never touch
// Pixi types directly — they hand World an EntityInit describing the entity
// shape (sprite/shape/container) and World materializes the right Pixi
// DisplayObject under the hood.

// biome-ignore lint/suspicious/noExplicitAny: vendored Pixi types are not bundled
type PixiAny = any

// biome-ignore lint/suspicious/noExplicitAny: vendored Pixi module is opaque
import * as PIXI from '../vendor/pixi.js/pixi.mjs' // eslint-disable-line @typescript-eslint/no-unused-vars
// biome-ignore lint/suspicious/noExplicitAny: see above
const pixi: any = PIXI

export interface SpriteInit {
  kind: 'sprite'
  src?: string
  /** Sprite-sheet grid as [cols, rows]; when set, frame controls which cell. */
  frames?: [number, number]
  frame?: number
  /** Anchor as [x, y] in 0..1; default [0.5, 0.5]. */
  anchor?: [number, number]
}

export interface ShapeInit {
  kind: 'shape'
  shape: 'rect' | 'circle' | 'polygon' | 'line'
  width?: number
  height?: number
  radius?: number
  points?: number[]
  fill?: number
  stroke?: number
  strokeWidth?: number
}

export interface ContainerInit {
  kind: 'container'
}

export interface ShapePart {
  shape: 'rect' | 'circle' | 'polygon'
  x?: number
  y?: number
  rotation?: number
  width?: number
  height?: number
  radius?: number
  points?: number[]
  fill?: number
  stroke?: number
  strokeWidth?: number
  alpha?: number
}

export interface CompositeInit {
  kind: 'composite'
  parts: ShapePart[]
}

export type EntityVisualInit = SpriteInit | ShapeInit | ContainerInit | CompositeInit

export interface EntityInit extends Partial<Pick<Entity, 'id' | 'tag' | 'group' | 'x' | 'y' | 'z' | 'vx' | 'vy' | 'rotation' | 'scale' | 'alpha' | 'tint'>> {
  visual: EntityVisualInit
}

export interface Entity {
  readonly id: string
  tag?: string
  group?: string
  x: number
  y: number
  z: number
  vx: number
  vy: number
  rotation: number
  scale: number
  alpha: number
  tint: number | null
  /** Free-form per-entity state for director / controllers. */
  data: Map<string, unknown>
  /** Opaque Pixi display object. Engine internals only. */
  display: PixiAny
  dead: boolean
  destroy(): void
}

export interface World {
  readonly app: PixiAny
  spawn(init: EntityInit): Entity
  destroy(e: Entity): void
  byId(id: string): Entity | undefined
  byTag(tag: string): Iterable<Entity>
  byGroup(group: string): Iterable<Entity>
  readonly entities: ReadonlyArray<Entity>
  /** Resolve a sprite-sheet src+frame into a Pixi Texture for engine internals. */
  loadTexture(src: string): Promise<PixiAny>
  /**
   * Repaint a composite entity's display with a fresh set of parts. The
   * existing child Graphics are destroyed and rebuilt; the entity's
   * transform stays intact so it stays in place. Used to recolor entities
   * on theme change without respawning them.
   */
  repaintComposite(entity: Entity, parts: ShapePart[]): void
  /** Repaint a single-shape entity (rect/circle/polygon) with new props. */
  repaintShape(entity: Entity, init: ShapeInit): void
  flush(): void
}

export interface WorldOptions {
  canvas: HTMLCanvasElement
  width: number
  height: number
  background: number
  pixelRatio: number
  renderer?: 'auto' | 'webgl' | 'canvas'
}

let nextId = 1

export async function createWorld(opts: WorldOptions): Promise<World> {
  // Drop Pixi's events plugin. Our render path doesn't need Pixi pointer
  // events (the arcade InputBus owns input); the plugin's renderer-init
  // hook trips on shadow-DOM-hosted canvases when its domElement target
  // is something other than a live HTMLCanvasElement, and it surfaces as
  // a hard `domElement.addEventListener is not a function` at mount.
  // Idempotent — re-removing is a no-op.
  try { pixi.extensions.remove(pixi.EventSystem) } catch { /* already removed */ }
  // Pixi's WebGL init wants a live HTMLCanvasElement available on the document;
  // canvases living in a closed shadow root sometimes fail getContext detection.
  // Init Pixi on its own canvas, then swap the original placeholder for Pixi's.
  const app = new pixi.Application()
  await app.init({
    width: opts.width,
    height: opts.height,
    background: opts.background,
    resolution: opts.pixelRatio,
    antialias: true,
    autoStart: false,
    eventFeatures: { move: false, click: false, wheel: false, globalMove: false },
    preference: 'webgl',
  })
  const pixiCanvas = app.canvas as HTMLCanvasElement
  pixiCanvas.setAttribute('part', 'canvas')
  pixiCanvas.style.cssText = 'display:block;width:100%;height:100%;touch-action:none'
  opts.canvas.replaceWith(pixiCanvas)

  const entities: Entity[] = []
  const byId = new Map<string, Entity>()
  const byTag = new Map<string, Set<Entity>>()
  const byGroup = new Map<string, Set<Entity>>()

  function index(e: Entity) {
    byId.set(e.id, e)
    if (e.tag) {
      let s = byTag.get(e.tag)
      if (!s) { s = new Set(); byTag.set(e.tag, s) }
      s.add(e)
    }
    if (e.group) {
      let s = byGroup.get(e.group)
      if (!s) { s = new Set(); byGroup.set(e.group, s) }
      s.add(e)
    }
  }

  function deindex(e: Entity) {
    byId.delete(e.id)
    if (e.tag) byTag.get(e.tag)?.delete(e)
    if (e.group) byGroup.get(e.group)?.delete(e)
  }

  function makeDisplay(v: EntityVisualInit): PixiAny {
    if (v.kind === 'shape') {
      const g = new pixi.Graphics()
      paintShape(g, v)
      return g
    }
    if (v.kind === 'sprite') {
      const sprite = new pixi.Sprite()
      const ax = v.anchor?.[0] ?? 0.5
      const ay = v.anchor?.[1] ?? 0.5
      sprite.anchor.set(ax, ay)
      return sprite
    }
    if (v.kind === 'composite') {
      const container = new pixi.Container()
      for (const part of v.parts) {
        const g = new pixi.Graphics()
        if (part.x !== undefined) g.x = part.x
        if (part.y !== undefined) g.y = part.y
        if (part.rotation !== undefined) g.rotation = part.rotation
        if (part.alpha !== undefined) g.alpha = part.alpha
        paintPart(g, part)
        container.addChild(g)
      }
      return container
    }
    return new pixi.Container()
  }

  function paintPart(g: PixiAny, p: ShapePart) {
    g.clear()
    const fill = p.fill ?? 0xffffff
    if (p.shape === 'rect') {
      const w = p.width ?? 4
      const h = p.height ?? 4
      g.rect(-w / 2, -h / 2, w, h).fill(fill)
    } else if (p.shape === 'circle') {
      g.circle(0, 0, p.radius ?? 3).fill(fill)
    } else if (p.shape === 'polygon' && p.points) {
      g.poly(p.points).fill(fill)
    }
    if (p.stroke !== undefined) g.stroke({ color: p.stroke, width: p.strokeWidth ?? 1 })
  }

  function paintShape(g: PixiAny, v: ShapeInit) {
    g.clear()
    const fill = v.fill ?? 0xffffff
    if (v.shape === 'rect') {
      const w = v.width ?? 10
      const h = v.height ?? 10
      g.rect(-w / 2, -h / 2, w, h).fill(fill)
      if (v.stroke !== undefined) g.stroke({ color: v.stroke, width: v.strokeWidth ?? 1 })
    } else if (v.shape === 'circle') {
      g.circle(0, 0, v.radius ?? 5).fill(fill)
      if (v.stroke !== undefined) g.stroke({ color: v.stroke, width: v.strokeWidth ?? 1 })
    } else if (v.shape === 'polygon' && v.points) {
      g.poly(v.points).fill(fill)
      if (v.stroke !== undefined) g.stroke({ color: v.stroke, width: v.strokeWidth ?? 1 })
    } else if (v.shape === 'line' && v.points) {
      const [x0, y0, ...rest] = v.points
      g.moveTo(x0 ?? 0, y0 ?? 0)
      for (let i = 0; i < rest.length; i += 2) g.lineTo(rest[i] ?? 0, rest[i + 1] ?? 0)
      g.stroke({ color: v.stroke ?? fill, width: v.strokeWidth ?? 1 })
    }
  }

  const textureCache = new Map<string, PixiAny>()
  const texturePending = new Map<string, Promise<PixiAny>>()

  async function loadTexture(src: string): Promise<PixiAny> {
    const cached = textureCache.get(src)
    if (cached) return cached
    const inflight = texturePending.get(src)
    if (inflight) return inflight
    const p = pixi.Assets.load(src).then((tex: PixiAny) => {
      textureCache.set(src, tex)
      texturePending.delete(src)
      return tex
    })
    texturePending.set(src, p)
    return p
  }

  function spawn(init: EntityInit): Entity {
    const display = makeDisplay(init.visual)
    const id = init.id ?? `e${nextId++}`
    const entity: Entity = {
      id,
      tag: init.tag,
      group: init.group,
      x: init.x ?? 0,
      y: init.y ?? 0,
      z: init.z ?? 0,
      vx: init.vx ?? 0,
      vy: init.vy ?? 0,
      rotation: init.rotation ?? 0,
      scale: init.scale ?? 1,
      alpha: init.alpha ?? 1,
      tint: init.tint ?? null,
      data: new Map(),
      display,
      dead: false,
      destroy() {
        if (entity.dead) return
        entity.dead = true
      },
    }
    display.x = entity.x
    display.y = entity.y
    display.rotation = entity.rotation
    display.scale.set(entity.scale)
    display.alpha = entity.alpha
    if (entity.tint !== null && 'tint' in display) display.tint = entity.tint
    app.stage.addChild(display)
    entities.push(entity)
    index(entity)
    if (init.visual.kind === 'sprite' && init.visual.src) {
      void loadTexture(init.visual.src).then((tex) => {
        if (entity.dead) return
        display.texture = tex
        if (init.visual.kind === 'sprite' && init.visual.frames) {
          applyFrame(display, tex, init.visual.frames, init.visual.frame ?? 0)
        }
      })
    }
    return entity
  }

  function destroy(e: Entity) { e.destroy() }

  function flush() {
    let writeIdx = 0
    for (let i = 0; i < entities.length; i++) {
      const e = entities[i]!
      if (e.dead) {
        deindex(e)
        app.stage.removeChild(e.display)
        e.display.destroy?.()
      } else {
        e.display.x = e.x
        e.display.y = e.y
        e.display.rotation = e.rotation
        e.display.scale.set(e.scale)
        e.display.alpha = e.alpha
        e.display.zIndex = e.z
        if (e.tint !== null && 'tint' in e.display) e.display.tint = e.tint
        entities[writeIdx++] = e
      }
    }
    entities.length = writeIdx
    app.stage.sortableChildren = true
  }

  function repaintComposite(entity: Entity, parts: ShapePart[]) {
    const container = entity.display
    if (!container || !container.removeChildren) return
    const old = container.removeChildren()
    for (const child of old) { try { child.destroy?.() } catch { /* already gone */ } }
    for (const part of parts) {
      const g = new pixi.Graphics()
      if (part.x !== undefined) g.x = part.x
      if (part.y !== undefined) g.y = part.y
      if (part.rotation !== undefined) g.rotation = part.rotation
      if (part.alpha !== undefined) g.alpha = part.alpha
      paintPart(g, part)
      container.addChild(g)
    }
  }

  function repaintShape(entity: Entity, init: ShapeInit) {
    const g = entity.display
    if (!g || !g.clear) return
    paintShape(g, init)
  }

  return {
    app,
    spawn,
    destroy,
    byId: (id) => byId.get(id),
    byTag: (tag) => byTag.get(tag) ?? new Set<Entity>(),
    byGroup: (g) => byGroup.get(g) ?? new Set<Entity>(),
    entities,
    loadTexture,
    repaintComposite,
    repaintShape,
    flush,
  }
}

function applyFrame(sprite: PixiAny, baseTex: PixiAny, frames: [number, number], frame: number) {
  const [cols, rows] = frames
  const fw = baseTex.width / cols
  const fh = baseTex.height / rows
  const cx = frame % cols
  const cy = Math.floor(frame / cols) % rows
  const rect = new pixi.Rectangle(cx * fw, cy * fh, fw, fh)
  sprite.texture = new pixi.Texture({ source: baseTex.source, frame: rect })
}
