// @ship/arcade-engine — minimal 2D arcade engine.
//
// Public surface only. PixiJS is an implementation detail — the engine
// imports it (the `pixi.js` npm dep) directly inside world/particles.
// Consumers never pass Pixi in or see Pixi types; they get a typed Stage handle.

export { createStage, type Stage, type StageOptions, type StageState } from './stage'
export { type Entity, type EntityInit, type EntityVisualInit, type SpriteInit, type ShapeInit, type ContainerInit, type CompositeInit, type ShapePart, type World } from './world'
export { type CollideSystem, type CollideEvent, type ColliderSpec } from './collide'
export { type InputBus, type Phase } from './input'
export { type AudioGraph, type Bus, type PlayOptions } from './audio'
export { type Palette } from './palette'
export { type ParticleSystem, type Emitter, type EmitterSpec } from './particles'
export { type Loop, type LoopOptions } from './loop'
