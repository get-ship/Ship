// Public API surface for @sail/runtime.
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Public API.
// Everything not exported here is internal and may change.

export { signal, derived, effect, batch, untrack } from './signal'
export type { Signal, Derived } from './signal'

export {
  bindText,
  bindShow,
  bindAttr,
  bindClass,
  bindProp,
  bindValue,
  bindChecked,
} from './bind'
export type { Disposer } from './bind'

export { on, matchesKey } from './on'
export type { OnOptions, Modifier } from './on'

export { applyPatch, applyPatches, tokenizePath } from './patch'
export type { Patch, PatchOp, PathToken } from './patch'

// Remaining modules (component, for, slot, data, morph, decode) land
// per the implementation phasing in 03-runtime-architecture.md.
// Import sites stay stable.
