// Pooled particle emitters. Each emitter owns a Pixi Container of pre-allocated
// Graphics nodes (circles); spawning marks one alive and (re)initializes its
// kinematics. Per step: integrate, fade size/alpha, recycle when dead.

// biome-ignore lint/suspicious/noExplicitAny: vendored Pixi types are not bundled
type PixiAny = any

import * as PIXI from 'pixi.js'
// biome-ignore lint/suspicious/noExplicitAny: vendored Pixi module is opaque
const pixi: any = PIXI

export interface EmitterSpec {
  kind: 'radial' | 'cone' | 'line' | 'explosion'
  rate: number
  burstCount: number
  lifetime: number
  spread: number
  direction: number
  speed: number
  speedJitter: number
  gravity: number
  color: number
  sizeStart: number
  sizeEnd: number
  max: number
}

export interface Emitter {
  x: number
  y: number
  spec: EmitterSpec
  burst(n?: number): void
  setColor(color: number): void
  destroy(): void
}

interface Particle {
  alive: boolean
  age: number
  life: number
  x: number
  y: number
  vx: number
  vy: number
  display: PixiAny
}

export interface ParticleSystem {
  add(world: WorldHandle, spec: EmitterSpec, x: number, y: number): Emitter
  step(dt: number): void
  destroy(): void
}

interface WorldHandle {
  app: PixiAny
}

function spawnParticle(p: Particle, em: Emitter) {
  p.alive = true
  p.age = 0
  p.life = em.spec.lifetime / 1000
  p.x = em.x
  p.y = em.y
  let angle = (em.spec.direction * Math.PI) / 180
  if (em.spec.kind === 'radial' || em.spec.kind === 'explosion') {
    angle = Math.random() * Math.PI * 2
  } else if (em.spec.kind === 'cone') {
    const half = (em.spec.spread * Math.PI) / 360
    angle += (Math.random() * 2 - 1) * half
  }
  const jitter = 1 + (Math.random() * 2 - 1) * em.spec.speedJitter
  const speed = em.spec.speed * jitter * (em.spec.kind === 'explosion' ? 1 + Math.random() : 1)
  p.vx = Math.cos(angle) * speed
  p.vy = Math.sin(angle) * speed
  p.display.x = p.x
  p.display.y = p.y
  p.display.alpha = 1
  p.display.scale.set(1)
  p.display.visible = true
}

export function createParticles(): ParticleSystem {
  const emitters: { em: Emitter; particles: Particle[]; container: PixiAny; spawnAccum: number }[] = []

  function makeParticle(spec: EmitterSpec, container: PixiAny): Particle {
    const g = new pixi.Graphics()
    g.circle(0, 0, spec.sizeStart).fill(spec.color)
    g.visible = false
    container.addChild(g)
    return { alive: false, age: 0, life: 0, x: 0, y: 0, vx: 0, vy: 0, display: g }
  }

  return {
    add(world, spec, x, y) {
      const container = new pixi.Container()
      world.app.stage.addChild(container)
      const particles: Particle[] = []
      for (let i = 0; i < spec.max; i++) particles.push(makeParticle(spec, container))
      const slot = { em: null as unknown as Emitter, particles, container, spawnAccum: 0 }
      const em: Emitter = {
        x, y, spec,
        burst(n) {
          const count = n ?? spec.burstCount
          let made = 0
          for (let i = 0; i < particles.length && made < count; i++) {
            if (!particles[i]!.alive) { spawnParticle(particles[i]!, em); made++ }
          }
        },
        setColor(color) {
          spec.color = color
          for (const p of particles) {
            p.display.clear()
            p.display.circle(0, 0, spec.sizeStart).fill(color)
          }
        },
        destroy() {
          const i = emitters.indexOf(slot)
          if (i >= 0) emitters.splice(i, 1)
          container.destroy?.({ children: true })
        },
      }
      slot.em = em
      emitters.push(slot)
      return em
    },
    step(dt) {
      for (const slot of emitters) {
        const { em, particles } = slot
        const spec = em.spec
        if (spec.rate > 0) {
          slot.spawnAccum += spec.rate * dt
          while (slot.spawnAccum >= 1) {
            for (let i = 0; i < particles.length; i++) {
              if (!particles[i]!.alive) { spawnParticle(particles[i]!, em); break }
            }
            slot.spawnAccum -= 1
          }
        }
        for (const p of particles) {
          if (!p.alive) continue
          p.age += dt
          if (p.age >= p.life) { p.alive = false; p.display.visible = false; continue }
          p.vy += spec.gravity * dt
          p.x += p.vx * dt
          p.y += p.vy * dt
          const t = p.age / p.life
          const size = spec.sizeStart + (spec.sizeEnd - spec.sizeStart) * t
          const s = spec.sizeStart > 0 ? size / spec.sizeStart : 0
          p.display.x = p.x
          p.display.y = p.y
          p.display.scale.set(Math.max(0, s))
          p.display.alpha = 1 - t
        }
      }
    },
    destroy() {
      for (const slot of [...emitters]) slot.em.destroy()
      emitters.length = 0
    },
  }
}
