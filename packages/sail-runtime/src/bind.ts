// Reactive DOM bindings. Each binding subscribes to its dependency
// signals and applies updates to one DOM node. Disposers returned
// for cleanup wiring (Sail's component wrapper chains them).
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Bindings.

import { effect, type Signal } from './signal'

/**
 * Cleanup function returned by every `bind*` helper and `effect`. Calling
 * it detaches the binding's listeners and stops further updates.
 */
export type Disposer = () => void

/**
 * Bind reactive text content to a node. Reads `get()` once eagerly and
 * again on every tracked-dependency change; the result is coerced to a
 * string and written to `node.textContent`. `null` and `undefined`
 * become `''` so missing values don't leak the literal text.
 *
 * Compiler target: `<el>{expr}</el>` interpolations.
 *
 * @param node Any DOM node with a `textContent` setter.
 * @param get Reactive read of the value to display.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const greeting = signal('hi')
 * bindText(span, () => greeting())   // span text follows greeting()
 * ```
 *
 * @see bindAttr, bindProp for non-text targets.
 */
export function bindText(node: Node, get: () => unknown): Disposer {
  return effect(() => {
    const v = get()
    node.textContent = v == null ? '' : String(v)
  })
}

/**
 * Toggle visibility based on a truthiness check. Falsy → `display:none`;
 * truthy → restores the previous `display` value (preserving any inline
 * value that was there before the binding hid the element).
 *
 * Use `bindShow` when the element should stay in the DOM (preserves
 * focus, scroll, child state); use `bindIf` when the branch should be
 * torn down and rebuilt.
 *
 * @param el Element whose `style.display` is toggled.
 * @param get Reactive predicate; truthy → visible, falsy → hidden.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const expanded = signal(false)
 * bindShow(panel, () => expanded())
 * ```
 *
 * @see bindIf for branch swap; bindClass to toggle visibility via CSS.
 */
export function bindShow(el: HTMLElement, get: () => unknown): Disposer {
  let prev: string | null = null
  return effect(() => {
    if (get()) {
      if (prev !== null) el.style.display = prev
      prev = null
    } else {
      if (prev === null) prev = el.style.display
      el.style.display = 'none'
    }
  })
}

/**
 * Bind a reactive value to a DOM attribute, with attribute-presence
 * semantics built in.
 *
 * Omission rules — chosen so authors write the obvious thing and get
 * correct DOM/ARIA semantics without thinking about absence:
 *   - `null`, `undefined`, `false`, `''` → remove the attribute
 *   - `true` → presence-only (`disabled`, `required`, ...)
 *   - anything else → `setAttribute(name, String(value))`
 *
 * The empty-string case matters for ARIA: `<input aria-label="">`
 * overrides the accessible-name computation, so silently writing it
 * is a real divergence. Removing empty strings means
 * `aria-label={maybeLabel}` does the right thing in either case.
 *
 * @param el Element whose attribute is mutated.
 * @param name Attribute name (kebab-case, as written in HTML).
 * @param get Reactive read of the attribute value.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const busy = signal(false)
 * bindAttr(btn, 'aria-busy', () => busy())
 * busy.set(true)   // <button aria-busy>
 * busy.set(false)  // <button>
 * ```
 *
 * @see bindProp when binding to a typed JS property; bindClass for class
 *   toggles.
 */
export function bindAttr(el: Element, name: string, get: () => unknown): Disposer {
  return effect(() => {
    const v = get()
    if (v == null || v === false || v === '') {
      el.removeAttribute(name)
    } else if (v === true) {
      el.setAttribute(name, '')
    } else {
      el.setAttribute(name, String(v))
    }
  })
}

/**
 * Toggle a single class on an element. Truthy → class added; falsy →
 * class removed. Other classes already on the element are untouched.
 *
 * Compiler target: `class:active={cond}` template syntax.
 *
 * @param el Element to toggle the class on.
 * @param name Class name (no leading `.`).
 * @param get Reactive predicate.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const isActive = signal(false)
 * bindClass(tab, 'active', () => isActive())
 * ```
 *
 * @see bindAttr for general attribute binding; bindShow for visibility.
 */
export function bindClass(el: Element, name: string, get: () => unknown): Disposer {
  return effect(() => {
    if (get()) el.classList.add(name)
    else el.classList.remove(name)
  })
}

/**
 * Bind a value to a JS property of an element (not an attribute). Use
 * this when the target is a typed property — arrays, objects, instances
 * — so the value flows through without an `attribute → string → parse`
 * round-trip.
 *
 * The compiler emits `bindProp` whenever the right-hand side is an
 * expression of non-string type assigned to a custom-element prop:
 * `<my-component items={list}>`.
 *
 * @param el Element to write the property on.
 * @param prop Property name (camelCase, as on the JS object).
 * @param get Reactive read of the property value.
 * @returns Disposer that stops the binding.
 *
 * @example
 * ```ts
 * const items = signal<Row[]>([])
 * bindProp(grid, 'items', () => items())
 * ```
 *
 * @see bindAttr for HTML attributes; bindValue for two-way input binding.
 */
export function bindProp<T extends Element>(
  el: T,
  prop: string,
  get: () => unknown,
): Disposer {
  return effect(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    ;(el as any)[prop] = get()
  })
}

/**
 * Two-way bind a string signal to an `<input>` / `<textarea>` /
 * `<select>` element. Initial render and signal-driven updates flow
 * signal → element; user edits flow element → signal via the `input`
 * event. Setting the same value re-checks identity to avoid moving the
 * caret.
 *
 * Compiler target: `bind:value={state}` template syntax.
 *
 * @param el Form control whose `.value` is two-way bound.
 * @param sig String signal that owns the value.
 * @returns Disposer that stops the binding and removes the listener.
 *
 * @example
 * ```ts
 * const draft = signal('')
 * bindValue(input, draft)
 * ```
 *
 * @see bindChecked for boolean controls; bindProp for one-way prop binding.
 */
export function bindValue(
  el: HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement,
  sig: Signal<string>,
): Disposer {
  const dispose = effect(() => {
    const v = sig()
    if (el.value !== v) el.value = v
  })
  const onInput = () => sig.set(el.value)
  el.addEventListener('input', onInput)
  return () => {
    dispose()
    el.removeEventListener('input', onInput)
  }
}

/**
 * Two-way bind a boolean signal to a checkbox or radio input. Signal
 * writes set `el.checked`; the `change` event writes back into the
 * signal.
 *
 * Compiler target: `bind:checked={state}` on `<input type="checkbox">`
 * or `<input type="radio">`.
 *
 * @param el Checkbox or radio input.
 * @param sig Boolean signal that owns the checked state.
 * @returns Disposer that stops the binding and removes the listener.
 *
 * @example
 * ```ts
 * const agreed = signal(false)
 * bindChecked(checkbox, agreed)
 * ```
 *
 * @see bindValue for text-valued inputs.
 */
export function bindChecked(
  el: HTMLInputElement,
  sig: Signal<boolean>,
): Disposer {
  const dispose = effect(() => {
    const v = sig()
    if (el.checked !== v) el.checked = v
  })
  const onChange = () => sig.set(el.checked)
  el.addEventListener('change', onChange)
  return () => {
    dispose()
    el.removeEventListener('change', onChange)
  }
}
