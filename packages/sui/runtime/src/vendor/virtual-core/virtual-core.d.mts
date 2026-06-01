// VENDORED TYPES — @tanstack/virtual-core 3.16.1, MIT (see ./LICENSE).
// Vendored 2026-06-01.
//
// Self-contained: derived from the tarball's `dist/esm/index.d.ts`, with the
// two `./utils.js` re-exports (`approxEqual`/`debounce`/`memo`/`notUndefined`
// and the `NoInfer`/`PartialKeys` type aliases) INLINED so this file resolves
// with no sibling `.d.ts` dependencies.
//
// This is a `.d.mts` so that `import … from './virtual-core.mjs'` (in
// `../../virtual.ts`, under `moduleResolution: Bundler`) resolves to it by
// extension-matched sibling lookup (`.mjs` → `.d.mts`) — the same mechanism
// `packages/arcade-engine/vendor/pixi.js/` relies on. The top-level `export`s
// below ARE the module's public surface; no ambient `declare module` needed.
//
// DO NOT EDIT by hand. Regenerate from the pinned upstream `dist/esm/index.d.ts`
// when bumping the version.

// --- inlined from dist/esm/utils.d.ts -------------------------------------
export type NoInfer<A extends any> = [A][A extends any ? 0 : never]
export type PartialKeys<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>
export declare function memo<TDeps extends ReadonlyArray<any>, TResult>(
  getDeps: () => [...TDeps],
  fn: (...args: NoInfer<[...TDeps]>) => TResult,
  opts: {
    key: false | string
    debug?: () => boolean
    onChange?: (result: TResult) => void
    initialDeps?: TDeps
    skipInitialOnChange?: boolean
  },
): {
  (): TResult
  updateDeps(newDeps: [...TDeps]): void
}
export declare function notUndefined<T>(value: T | undefined, msg?: string): T
export declare const approxEqual: (a: number, b: number) => boolean
export declare const debounce: (
  targetWindow: Window & typeof globalThis,
  fn: Function,
  ms: number,
) => (this: any, ...args: Array<any>) => void

// --- from dist/esm/index.d.ts (verbatim, sans the ./utils.js re-export) ----
export declare const _resetIOSDetectionForTests: () => void
type ScrollDirection = 'forward' | 'backward'
type ScrollAlignment = 'start' | 'center' | 'end' | 'auto'
type ScrollBehavior = 'auto' | 'smooth' | 'instant'
type ScrollAnchor = 'start' | 'end'
type FollowOnAppend = boolean | ScrollBehavior
export interface ScrollToOptions {
  align?: ScrollAlignment
  behavior?: ScrollBehavior
}
type ScrollToOffsetOptions = ScrollToOptions
type ScrollToIndexOptions = ScrollToOptions
type ScrollToEndOptions = Pick<ScrollToOptions, 'behavior'>
export interface Range {
  startIndex: number
  endIndex: number
  overscan: number
  count: number
}
type Key = number | string | bigint
export interface VirtualItem {
  key: Key
  index: number
  start: number
  end: number
  size: number
  lane: number
}
export interface Rect {
  width: number
  height: number
}
export declare const defaultKeyExtractor: (index: number) => number
export declare const defaultRangeExtractor: (range: Range) => number[]
export declare const observeElementRect: <T extends Element>(
  instance: Virtualizer<T, any>,
  cb: (rect: Rect) => void,
) => (() => void) | undefined
export declare const observeWindowRect: (
  instance: Virtualizer<Window, any>,
  cb: (rect: Rect) => void,
) => (() => void) | undefined
type ObserveOffsetCallBack = (offset: number, isScrolling: boolean) => void
export declare const observeElementOffset: <T extends Element>(
  instance: Virtualizer<T, any>,
  cb: ObserveOffsetCallBack,
) => (() => void) | undefined
export declare const observeWindowOffset: (
  instance: Virtualizer<Window, any>,
  cb: ObserveOffsetCallBack,
) => (() => void) | undefined
export declare const measureElement: <TItemElement extends Element>(
  element: TItemElement,
  entry: ResizeObserverEntry | undefined,
  instance: Virtualizer<any, TItemElement>,
) => number
export declare const windowScroll: <T extends Window>(
  offset: number,
  options: {
    adjustments?: number
    behavior?: ScrollBehavior
  },
  instance: Virtualizer<T, any>,
) => void
export declare const elementScroll: <T extends Element>(
  offset: number,
  options: {
    adjustments?: number
    behavior?: ScrollBehavior
  },
  instance: Virtualizer<T, any>,
) => void
type LaneAssignmentMode = 'estimate' | 'measured'
export interface VirtualizerOptions<
  TScrollElement extends Element | Window,
  TItemElement extends Element,
> {
  count: number
  getScrollElement: () => TScrollElement | null
  estimateSize: (index: number) => number
  scrollToFn: (
    offset: number,
    options: {
      adjustments?: number
      behavior?: ScrollBehavior
    },
    instance: Virtualizer<TScrollElement, TItemElement>,
  ) => void
  observeElementRect: (
    instance: Virtualizer<TScrollElement, TItemElement>,
    cb: (rect: Rect) => void,
  ) => void | (() => void)
  observeElementOffset: (
    instance: Virtualizer<TScrollElement, TItemElement>,
    cb: ObserveOffsetCallBack,
  ) => void | (() => void)
  debug?: boolean
  initialRect?: Rect
  onChange?: (
    instance: Virtualizer<TScrollElement, TItemElement>,
    sync: boolean,
  ) => void
  measureElement?: (
    element: TItemElement,
    entry: ResizeObserverEntry | undefined,
    instance: Virtualizer<TScrollElement, TItemElement>,
  ) => number
  overscan?: number
  horizontal?: boolean
  paddingStart?: number
  paddingEnd?: number
  scrollPaddingStart?: number
  scrollPaddingEnd?: number
  initialOffset?: number | (() => number)
  getItemKey?: (index: number) => Key
  rangeExtractor?: (range: Range) => Array<number>
  scrollMargin?: number
  gap?: number
  indexAttribute?: string
  initialMeasurementsCache?: Array<VirtualItem>
  lanes?: number
  anchorTo?: ScrollAnchor
  followOnAppend?: FollowOnAppend
  scrollEndThreshold?: number
  isScrollingResetDelay?: number
  useScrollendEvent?: boolean
  enabled?: boolean
  isRtl?: boolean
  useAnimationFrameWithResizeObserver?: boolean
  laneAssignmentMode?: LaneAssignmentMode
}
export declare class Virtualizer<
  TScrollElement extends Element | Window,
  TItemElement extends Element,
> {
  private unsubs
  options: Required<VirtualizerOptions<TScrollElement, TItemElement>>
  scrollElement: TScrollElement | null
  targetWindow: (Window & typeof globalThis) | null
  isScrolling: boolean
  private scrollState
  measurementsCache: Array<VirtualItem>
  private _flatMeasurements
  private itemSizeCache
  private itemSizeCacheVersion
  private laneAssignments
  private pendingMin
  private prevLanes
  private lanesChangedFlag
  private lanesSettling
  private pendingScrollAnchor
  scrollRect: Rect | null
  scrollOffset: number | null
  scrollDirection: ScrollDirection | null
  private scrollAdjustments
  private _iosDeferredAdjustment
  private _iosTouching
  private _iosJustTouchEnded
  private _iosTouchEndTimerId
  private _intendedScrollOffset
  shouldAdjustScrollPositionOnItemSizeChange:
    | undefined
    | ((
        item: VirtualItem,
        delta: number,
        instance: Virtualizer<TScrollElement, TItemElement>,
      ) => boolean)
  elementsCache: Map<Key, TItemElement>
  private now
  private observer
  range: {
    startIndex: number
    endIndex: number
  } | null
  constructor(opts: VirtualizerOptions<TScrollElement, TItemElement>)
  setOptions: (opts: VirtualizerOptions<TScrollElement, TItemElement>) => void
  private notify
  private applyScrollAdjustment
  private maybeNotify
  private cleanup
  _didMount: () => () => void
  _willUpdate: () => void
  private _flushIosDeferredIfReady
  private rafId
  private scheduleScrollReconcile
  private reconcileScroll
  private getSize
  private getScrollOffset
  private getFurthestMeasurement
  private getMeasurementOptions
  private getMeasurements
  calculateRange: {
    (): {
      startIndex: number
      endIndex: number
    } | null
    updateDeps(newDeps: [VirtualItem[], number, number, number]): void
  }
  getVirtualIndexes: {
    (): number[]
    updateDeps(
      newDeps: [
        (range: Range) => Array<number>,
        number,
        number,
        number | null,
        number | null,
      ],
    ): void
  }
  indexFromElement: (node: TItemElement) => number
  private shouldMeasureDuringScroll
  measureElement: (node: TItemElement | null) => void
  resizeItem: (index: number, size: number) => void
  getVirtualItems: {
    (): VirtualItem[]
    updateDeps(newDeps: [number[], VirtualItem[]]): void
  }
  getVirtualItemForOffset: (offset: number) => VirtualItem | undefined
  private getMaxScrollOffset
  private getVirtualDistanceFromEnd
  getDistanceFromEnd: () => number
  isAtEnd: (threshold?: number) => boolean
  getOffsetForAlignment: (
    toOffset: number,
    align: ScrollAlignment,
    itemSize?: number,
  ) => number
  getOffsetForIndex: (
    index: number,
    align?: ScrollAlignment,
  ) =>
    | readonly [number, 'auto']
    | readonly [number, 'start' | 'center' | 'end']
    | undefined
  scrollToOffset: (
    toOffset: number,
    { align, behavior }?: ScrollToOffsetOptions,
  ) => void
  scrollToIndex: (
    index: number,
    { align: initialAlign, behavior }?: ScrollToIndexOptions,
  ) => void
  scrollBy: (delta: number, { behavior }?: ScrollToOffsetOptions) => void
  scrollToEnd: ({ behavior }?: ScrollToEndOptions) => void
  getTotalSize: () => number
  takeSnapshot: () => Array<VirtualItem>
  private _scrollToOffset
  measure: () => void
}
