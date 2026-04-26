// Reactive DOM bindings. Each binding subscribes to its dependency
// signals and applies updates to one DOM node. Disposers returned
// for cleanup wiring (Sail's component wrapper chains them).
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Bindings.

import { effect, type Signal } from './signal'

export type Disposer = () => void

/**
 * Bind reactive text content to a node. Reads `get()` initially and
 * on every dep change, writes the result to `node.textContent`.
 *
 * Used for `<el>{expr}</el>` interpolations.
 */
export function bindText(node: Node, get: () => unknown): Disposer {
  return effect(() => {
    const v = get()
    node.textContent = v == null ? '' : String(v)
  })
}

/**
 * Toggle visibility based on a boolean expression. Falsy → display:none.
 *
 * Used for `{#if cond}…{/if}` blocks (a primitive form; the full
 * `{#if}` block reconciler lives in `for.ts` once it lands).
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
 * Bind any attribute. `null` / `false` → remove attribute;
 * `true` → empty attribute (presence-only); other → String(value).
 */
export function bindAttr(el: Element, name: string, get: () => unknown): Disposer {
  return effect(() => {
    const v = get()
    if (v == null || v === false) {
      el.removeAttribute(name)
    } else if (v === true) {
      el.setAttribute(name, '')
    } else {
      el.setAttribute(name, String(v))
    }
  })
}

/**
 * Bind a class name. Truthy → class added; falsy → removed.
 *
 * Used for `data-class:active={cond}` style bindings (Sail's syntax
 * is `class:active={cond}` in templates).
 */
export function bindClass(el: Element, name: string, get: () => unknown): Disposer {
  return effect(() => {
    if (get()) el.classList.add(name)
    else el.classList.remove(name)
  })
}

/**
 * Bind a property directly (vs an attribute). For non-string values
 * — arrays, objects, custom-element typed properties — this avoids
 * the JSON.stringify round-trip.
 *
 * Used for `<my-component items={list}>` where items is `Item[]`.
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
 * Two-way binding for form inputs. Reads from the signal initially +
 * on changes; writes back when the user edits.
 *
 * Used for `bind:value={state}` on inputs/textareas/selects, and
 * `bind:checked={state}` on checkboxes.
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

/** Two-way binding for checkbox / radio inputs. */
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
