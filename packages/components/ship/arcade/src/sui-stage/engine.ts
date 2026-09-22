// Re-export the engine boundary so the <sui-stage> controller stays
// package-local (relative imports only). Tests mock `@ship/arcade-engine`.
export {
  createStage,
  type CollideEvent,
  type Emitter,
  type EntityVisualInit,
  type Stage,
  type StageState,
} from '@ship/arcade-engine'
