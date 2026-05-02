/**
 * @ship/ui-runtime — public API surface.
 *
 * Everything re-exported from this module is part of ship-set's stable
 * runtime contract. Anything not re-exported here is internal and may
 * change without notice.
 *
 * Re-exports keep their original JSDoc — hover any symbol in your
 * editor to see the full docs.
 *
 * Surface map:
 *   - Reactivity:    `signal`, `derived`, `effect`, `batch`, `untrack`
 *                    (+ types `Signal`, `Derived`)
 *   - DOM bindings:  `bindText`, `bindShow`, `bindAttr`, `bindClass`,
 *                    `bindProp`, `bindValue`, `bindChecked`, `bindRawHtml`
 *                    (+ type `Disposer`)
 *   - Events:        `on`, `matchesKey` (+ types `OnOptions`, `Modifier`)
 *   - Patches:       `applyPatch`, `applyPatches`, `tokenizePath`
 *                    (+ types `Patch`, `PatchOp`, `PathToken`)
 *   - Control flow:  `bindFor`, `bindForSignal`, `bindIf`, `bindAwait`
 *                    (+ types `BindForOptions`, `RenderedItem`,
 *                    `BindIfOptions`, `BranchRender`, `BindAwaitOptions`)
 *   - HMR / SSR:     `morph` (+ type `MorphOptions`)
 *   - Slots:         `resolveSlots`, `snapshotChildren`, `slotFilled`
 *                    (+ type `SlotDecl`)
 *   - Components:    `defineComponent`, `camelOf`
 *                    (+ types `ComponentFactory`, `ComponentSchema`)
 *   - Prop decoders: `decodeString`, `decodeNumber`, `decodeBoolean`,
 *                    `decodeJson`, `decodeEnum` (+ type `Decoder`)
 *   - Data:          `data` (+ types `DataOpts`, `DataHandle`)
 *
 * See .artifacts/specs/ship-set/03-runtime-architecture.md § Public API.
 *
 * @packageDocumentation
 */

export { signal, derived, effect, batch, untrack } from './signal'
export type { Signal, Derived } from './signal'

export type { Ref } from './ref'

export { bindSlotEmpty } from './slot-empty'

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

export { state, debounce } from './state'
export type { State } from './state'

export { live, unwrap, unwrapOr } from './live'
export type { LiveState, LiveSignal, LiveSource, LiveOpts } from './live'

export { wireFormMutation } from './mutation'

export { bindWhileLoading, bindWhileError } from './while'
export type {
  BindWhileLoadingOptions,
  BindWhileErrorOptions,
} from './while'

export type { ShipSDK } from './ship'

export { route, navigate, defineRoutes } from './route'
export type { RouteSignals, RouteShape } from './route'

export { component, prop } from './component'
export type { PropDef, ComponentDef, ComponentSetupContext } from './component'

export type {
  ControllerDef,
  ControllerShape,
  ControllerHook,
  TemplateHook,
  TemplateHookAttrKind,
  TemplateHookContext,
  ViewScope,
  ReadonlyState,
  ReadonlyView,
  MutableState,
  SectionInit,
  DerivedSection,
  OnSection,
  UseSection,
  MountSection,
  UnmountSection,
} from './controller'

export { controller, resolveController, LIFECYCLE_KEY } from './controller-resolver'
export type {
  ControllerExport,
  ControllerLifecycle,
  ResolverContext,
  ResolvedBindings,
} from './controller-resolver'

export { view } from './view'
export type { ViewContext, ViewSetup, ViewDef } from './view'

export { useFilteredList, useSelection, useForm } from './hooks'
export type { FilteredListSource, UseFormOptions } from './hooks'

export {
  registerTemplateHook,
  getTemplateHook,
  clearRegistry,
  listRegisteredHooks,
} from './template-hooks'
export type { KbdShortcut, ShellContext } from './template-hooks'
// Importing the index ensures the three built-in hooks are registered
// at module load.
import './template-hooks'
// Register <sui-if>, <sui-for>, <sui-match> custom elements.
import './boundary-elements'

export { suiTonesSheet } from './styles/index'
