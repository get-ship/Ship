// Custom-element wrapper. The Sail compiler emits one
// `defineComponent('my-tag', factory)` call per .sail file. The factory
// is the user's script + binding wiring; defineComponent registers
// the wrapper class with `customElements.define`.
//
// See .artifacts/specs/sail/03-runtime-architecture.md § Component wrapper.

import type { Disposer } from './bind'
import type { Signal } from './signal'

/**
 * The factory the compiler emits per component. Receives the host
 * element + a place to attach prop signals; returns a cleanup function
 * that runs on disconnect.
 */
export type ComponentFactory = (
  host: HTMLElement,
  propSignals: Record<string, Signal<unknown>>,
) => Disposer

/**
 * Schema the compiler emits alongside the factory — drives prop decode
 * + observed-attributes registration.
 */
export interface ComponentSchema {
  /** Prop attribute names (kebab-case) the element observes. */
  observedAttrs: string[]
  /** Map prop attribute name → camelCase property name + decoder. */
  props: Record<
    string,
    {
      camel: string
      decode: (raw: string | null) => unknown
    }
  >
}

/**
 * Register a Sail component as a custom element. Idempotent — calling
 * twice with the same tag is a no-op (browser would otherwise throw).
 */
export function defineComponent(
  tag: string,
  schema: ComponentSchema,
  factory: ComponentFactory,
): void {
  if (customElements.get(tag)) return

  class SailElement extends HTMLElement {
    static get observedAttributes(): string[] {
      return schema.observedAttrs
    }

    private cleanup?: Disposer
    private propSignals: Record<string, Signal<unknown>> = {}

    connectedCallback() {
      // Mark for CSS scoping ([data-component="<tag>"]).
      this.dataset.component = tag
      // Run user factory; capture cleanup.
      this.cleanup = factory(this, this.propSignals)
    }

    disconnectedCallback() {
      try {
        this.cleanup?.()
      } catch (e) {
        console.error(`[sail:${tag}] cleanup threw:`, e)
      }
      this.cleanup = undefined
    }

    attributeChangedCallback(attrName: string, _old: string, value: string) {
      const meta = schema.props[attrName]
      if (!meta) return
      const sig = this.propSignals[meta.camel]
      if (sig) sig.set(meta.decode(value))
    }
  }

  customElements.define(tag, SailElement)
}

/**
 * Convert a kebab-case attribute name to camelCase prop name.
 *
 * `fetch-url` → `fetchUrl`
 * `min-chars` → `minChars`
 * `count`     → `count`
 */
export function camelOf(attr: string): string {
  return attr.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase())
}
