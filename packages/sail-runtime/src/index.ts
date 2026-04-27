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

export { bindFor, bindForSignal } from './for'
export type { BindForOptions, RenderedItem } from './for'

export { bindIf } from './if'
export type { BindIfOptions, BranchRender } from './if'

export { bindAwait } from './await'
export type { BindAwaitOptions } from './await'

export { bindRawHtml } from './raw-html'

export { morph } from './morph'
export type { MorphOptions } from './morph'

export { resolveSlots, snapshotChildren, slotFilled } from './slot'
export type { SlotDecl } from './slot'

export { defineComponent, camelOf } from './component'
export type { ComponentFactory, ComponentSchema } from './component'

export {
  decodeString,
  decodeNumber,
  decodeBoolean,
  decodeJson,
  decodeEnum,
} from './decode'
export type { Decoder } from './decode'

export { data } from './data'
export type { DataOpts, DataHandle } from './data'

// Remaining: morph.ts (vendored idiomorph) lands next per
// 03-runtime-architecture.md § Implementation phasing.
