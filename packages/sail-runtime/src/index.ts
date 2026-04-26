// Public API surface for @sail/runtime.
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Public API.
// Everything not exported here is internal and may change.

export { signal, derived, effect, batch, untrack } from './signal'
export type { Signal, Derived } from './signal'

// The remaining modules (component, bind, on, for, slot, patch, data,
// morph, decode) land per the implementation phasing in
// 03-runtime-architecture.md. Import sites stay stable.
