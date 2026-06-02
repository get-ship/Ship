import { describe, expect, it, vi } from 'vitest'
import { pixiMockModule } from './_pixi-mock.ts'

// index.ts re-exports the public surface. Almost everything is a type (erased
// at runtime); the one runtime value is createStage. Importing index pulls in
// stage → world → particles → the vendored Pixi bundle, so mock Pixi to keep
// the import cheap and DOM-free.
vi.mock('../vendor/pixi.js/pixi.mjs', () => pixiMockModule())

import * as engine from '../src/index.ts'

describe('index: public surface', () => {
  it('exports createStage as the sole runtime entry point', () => {
    expect(typeof engine.createStage).toBe('function')
  })

  it('does not leak Pixi or subsystem factories onto the public surface', () => {
    // The engine boundary hides Pixi and exposes only a typed Stage factory:
    // createWorld/createCollide/etc. are internal, not re-exported here.
    const runtimeKeys = Object.keys(engine).filter((k) => typeof (engine as Record<string, unknown>)[k] === 'function')
    expect(runtimeKeys).toEqual(['createStage'])
  })
})
