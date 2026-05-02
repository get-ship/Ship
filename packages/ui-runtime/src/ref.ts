// Refs — the SUI parallel of Svelte's `bind:this={el}` and React's
// `useRef<T>()`.
//
// A `Ref<T>` is just a `Signal<T | null>` with documentation framing.
// Authors declare a ref in their template via `bind:this={var}`; the
// compiler emits `Signal<T | null>` into the generated `Scope` so the
// paired controller has a typed handle on the rendered element. The
// runtime sets the value at attach time (views) or during template
// construction (components), and clears it on dispose.
//
// Refs are write-once-from-runtime, read-many-from-controller —
// authors should never call `.set()` on a ref themselves.

import type { Signal } from './signal'

/**
 * Typed handle to a single rendered element.
 *
 * Read inside `derived` / `effect` to subscribe to attach + dispose
 * transitions: the ref reads `null` before the runtime resolves the
 * element, and `null` again after the boundary unmounts. Until then
 * it returns the live element.
 *
 * Created indirectly: declare `bind:this={btn}` on an element and the
 * compiler emits `btn: Ref<HTMLButtonElement>` in `Scope`. Do not call
 * `.set()` on a ref from author code — the runtime owns the writes.
 */
export type Ref<T extends Element = HTMLElement> = Signal<T | null>
