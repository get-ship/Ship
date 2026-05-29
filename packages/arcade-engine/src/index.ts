// @ship/arcade-engine — minimal 2D arcade engine.
//
// Public surface only. PixiJS is a vendored implementation detail — the
// engine imports it directly inside stage/world/particles. Consumers never
// pass Pixi in or see Pixi types; they get a typed Stage handle.

export { createStage, type Stage, type StageOptions, type StageState } from './stage.ts'
export { type Entity, type EntityInit, type EntityVisualInit, type SpriteInit, type ShapeInit, type ContainerInit, type CompositeInit, type ShapePart, type World } from './world.ts'
export { type CollideSystem, type CollideEvent, type ColliderSpec } from './collide.ts'
export { type InputBus, type Phase } from './input.ts'
export { type AudioGraph, type Bus, type PlayOptions } from './audio.ts'
export { type Palette } from './palette.ts'
export { type ParticleSystem, type Emitter, type EmitterSpec } from './particles.ts'
export { type Loop, type LoopOptions } from './loop.ts'
