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
 *   - Control flow:  `bindFor`, `bindForSignal`, `bindIf`, `bindAwait`
 *                    (+ types `BindForOptions`, `RenderedItem`,
 *                    `BindIfOptions`, `BranchRender`, `BindAwaitOptions`)
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

export {
  createScope,
  createChildScope,
  withScope,
  onCleanup,
  currentScope,
} from './effect-scope'
export type { EffectScope } from './effect-scope'

// Dev-mode leak detector lives at `@ship/ui-runtime/dev` (separate
// subpath import so the wrappers don't ship in the production
// runtime bundle). See packages/ui-runtime/src/dev.ts.

export { shared, isShared, provide, useShared, pickShared } from './context'

export { ref, attachRef } from './ref'
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
  bindHtml,
} from './bind'
export type { Disposer } from './bind'

export { on, matchesKey } from './on'
export type { OnOptions, Modifier } from './on'

export {
  emit,
  expose,
  attachHostApi,
  bindNativeEvent,
  slotted,
  bindHostAttr,
} from './host-api'

export { bindFor, bindForSignal } from './for'
export type { BindForOptions, RenderedItem } from './for'

export { bindIf } from './if'
export type { BindIfOptions, BranchRender } from './if'

export { bindAwait } from './await'
export type { BindAwaitOptions } from './await'

export { bindRawHtml } from './raw-html'

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
} from './template-hooks/registry'
export type { KbdShortcut } from './template-hooks/kbd'
export type { ShellContext } from './template-hooks/shell'
// Built-in hook factories are loaded on demand via the
// `@ship/ui-runtime/template-hooks` subpath: the SUI compiler emits a
// side-effect import of that subpath into compiled views that contain
// `<use-*>` elements, so hook code only ships with views that use it.
// Register <sui-if>, <sui-for>, <sui-match> custom elements.
import './boundary-elements'

export { suiTonesSheet, suiBaseSheet, adoptSuiSheets } from './styles/index'

// F1 mount unification — hosts call mount(target, payload, opts) to put
// a view on screen. See .artifacts/design-f1-mount-unification.md.
export { mount } from './mount'
export type { ViewPayload, MountOptions, MountHandle } from './mount'

// Generated daemon ↔ runtime data contract. Authored by `cargo xtask
// gen-types` from Rust source; do not edit `generated.ts` by hand.
export type {
  AgentSummary,
  AgentsSummariesResponse,
  BranchSummary,
  CommitSummary,
  DiffFileView,
  DiffHunk,
  DiffLine,
  GitDiff,
  GitDiffArgs,
  GitOverview,
  LintFinding,
  LintViewRequest,
  ManifestRail,
  ParsedDiff,
  RailMeta,
  ResolveError,
  ResolvedView,
  StatusBuckets,
  StatusFile,
  ViewListEntry,
  ViewManifest,
  ViewSourceResponse,
  WriteDiagnostics,
  WriteManifestRequest,
  WriteResponse,
  WriteTemplateRequest,
} from './generated'
