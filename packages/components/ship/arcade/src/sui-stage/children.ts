// Parse light-DOM children of <sui-stage> into engine-friendly spec objects.
// Read-only; runs once at init. Dynamic markup changes after init are not
// observed (v0.1) — directors should spawn entities through stage.world.

export interface InputSpec {
  keys: string[]
  action: string
  repeat: boolean
}

export interface SfxSpec {
  src: string
  action: string
  volume: number
  pitchRange: [number, number]
  bus: 'sfx' | 'music' | 'ui'
}

export interface EmitterChildSpec {
  id: string | null
  kind: 'radial' | 'cone' | 'line' | 'explosion'
  x: number
  y: number
  rate: number
  burst: number
  lifetime: number
  spread: number
  direction: number
  speed: number
  speedJitter: number
  gravity: number
  color: string
  size: string
  max: number
  action: string
}

export interface ColliderSubSpec {
  kind: 'aabb' | 'circle'
  width: number
  height: number
  radius: number
  offsetX: number
  offsetY: number
  group: string
  collidesWith: string[]
  trigger: boolean
}

export interface EntityChildSpec {
  visual: 'sprite' | 'shape'
  src: string
  shape: 'rect' | 'circle' | 'polygon' | 'line'
  width: number
  height: number
  radius: number
  points: number[]
  fill: string
  stroke: string
  strokeWidth: number
  x: number
  y: number
  z: number
  vx: number
  vy: number
  rotation: number
  scale: number
  alpha: number
  tint: string
  anchor: [number, number]
  frames: [number, number] | undefined
  frame: number
  group: string
  tag: string
  collider: ColliderSubSpec | null
}

export interface ChildSpecs {
  inputs: InputSpec[]
  sfx: SfxSpec[]
  emitters: EmitterChildSpec[]
  entities: EntityChildSpec[]
}

const ANCHORS: Record<string, [number, number]> = {
  center: [0.5, 0.5],
  'top-left': [0, 0],
  'top-right': [1, 0],
  'top-center': [0.5, 0],
  'bottom-left': [0, 1],
  'bottom-right': [1, 1],
  'bottom-center': [0.5, 1],
  'middle-left': [0, 0.5],
  'middle-right': [1, 0.5],
}

function num(el: Element, attr: string, def: number): number {
  const raw = el.getAttribute(attr)
  if (raw === null || raw === '') return def
  const v = Number(raw)
  return Number.isFinite(v) ? v : def
}

function str(el: Element, attr: string, def: string): string {
  return el.getAttribute(attr) ?? def
}

function bool(el: Element, attr: string, def: boolean): boolean {
  const raw = el.getAttribute(attr)
  if (raw === null) return def
  return raw !== 'false' && raw !== '0'
}

function csv(s: string): string[] {
  return s.split(',').map((p) => p.trim()).filter(Boolean)
}

function parseAnchor(raw: string): [number, number] {
  if (raw in ANCHORS) return ANCHORS[raw]!
  const parts = raw.split(',').map((p) => Number(p.trim()))
  if (parts.length === 2 && Number.isFinite(parts[0]) && Number.isFinite(parts[1])) {
    return [parts[0]!, parts[1]!]
  }
  return [0.5, 0.5]
}

function parseFrames(raw: string): [number, number] | undefined {
  if (!raw) return undefined
  const m = /^(\d+)\s*x\s*(\d+)$/.exec(raw.trim())
  if (!m) return undefined
  return [Number(m[1]), Number(m[2])]
}

function parsePitchRange(raw: string): [number, number] {
  const parts = raw.split('..').map((p) => Number(p.trim()))
  const lo = Number.isFinite(parts[0]) ? parts[0]! : 1
  const hi = Number.isFinite(parts[1]) ? parts[1]! : lo
  return [lo, hi]
}

export function parseEmitterSize(raw: string): [number, number] {
  const parts = raw.split('..').map((p) => Number(p.trim()))
  const start = Number.isFinite(parts[0]) ? parts[0]! : 3
  const end = Number.isFinite(parts[1]) ? parts[1]! : 0
  return [start, end]
}

function readCollider(parent: Element): ColliderSubSpec | null {
  const c = parent.querySelector(':scope > sui-collider')
  if (!c) return null
  return {
    kind: (str(c, 'kind', 'aabb') as 'aabb' | 'circle'),
    width: num(c, 'width', 0),
    height: num(c, 'height', 0),
    radius: num(c, 'radius', 0),
    offsetX: num(c, 'offset-x', 0),
    offsetY: num(c, 'offset-y', 0),
    group: str(c, 'group', ''),
    collidesWith: csv(str(c, 'collides-with', '')),
    trigger: bool(c, 'trigger', false),
  }
}

function readEntity(el: Element, visual: 'sprite' | 'shape'): EntityChildSpec {
  return {
    visual,
    src: str(el, 'src', ''),
    shape: (str(el, 'kind', 'rect') as 'rect' | 'circle' | 'polygon' | 'line'),
    width: num(el, 'width', visual === 'shape' ? 10 : 0),
    height: num(el, 'height', visual === 'shape' ? 10 : 0),
    radius: num(el, 'radius', visual === 'shape' ? 5 : 0),
    points: str(el, 'points', '').split(/[\s,]+/).map(Number).filter(Number.isFinite),
    fill: str(el, 'fill', 'surface-fg'),
    stroke: str(el, 'stroke', ''),
    strokeWidth: num(el, 'stroke-width', 1),
    x: num(el, 'x', 0),
    y: num(el, 'y', 0),
    z: num(el, 'z', 0),
    vx: num(el, 'vx', 0),
    vy: num(el, 'vy', 0),
    rotation: num(el, 'rotation', 0),
    scale: num(el, 'scale', 1),
    alpha: num(el, 'alpha', 1),
    tint: str(el, 'tint', ''),
    anchor: parseAnchor(str(el, 'anchor', 'center')),
    frames: parseFrames(str(el, 'frames', '')),
    frame: num(el, 'frame', 0),
    group: str(el, 'group', ''),
    tag: str(el, 'tag', ''),
    collider: readCollider(el),
  }
}

export function readChildSpecs(host: Element): ChildSpecs {
  const out: ChildSpecs = { inputs: [], sfx: [], emitters: [], entities: [] }
  for (const el of host.querySelectorAll(':scope > sui-stage-input')) {
    out.inputs.push({
      keys: csv(str(el, 'map', '')),
      action: str(el, 'action', ''),
      repeat: bool(el, 'repeat', true),
    })
  }
  for (const el of host.querySelectorAll(':scope > sui-sfx')) {
    out.sfx.push({
      src: str(el, 'src', ''),
      action: str(el, 'on', ''),
      volume: num(el, 'volume', 1),
      pitchRange: parsePitchRange(str(el, 'pitch-range', '1..1')),
      bus: (str(el, 'bus', 'sfx') as 'sfx' | 'music' | 'ui'),
    })
  }
  for (const el of host.querySelectorAll(':scope > sui-emitter')) {
    out.emitters.push({
      id: el.getAttribute('id'),
      kind: (str(el, 'kind', 'radial') as 'radial' | 'cone' | 'line' | 'explosion'),
      x: num(el, 'x', 0),
      y: num(el, 'y', 0),
      rate: num(el, 'rate', 0),
      burst: num(el, 'burst', 16),
      lifetime: num(el, 'lifetime', 600),
      spread: num(el, 'spread', 360),
      direction: num(el, 'direction', -90),
      speed: num(el, 'speed', 120),
      speedJitter: num(el, 'speed-jitter', 0.3),
      gravity: num(el, 'gravity', 0),
      color: str(el, 'color', 'accent'),
      size: str(el, 'size', '3..0'),
      max: num(el, 'max', 64),
      action: str(el, 'on', ''),
    })
  }
  for (const el of host.querySelectorAll(':scope > sui-sprite')) out.entities.push(readEntity(el, 'sprite'))
  for (const el of host.querySelectorAll(':scope > sui-shape')) out.entities.push(readEntity(el, 'shape'))
  return out
}
