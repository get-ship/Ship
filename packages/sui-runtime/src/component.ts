// Custom-element wrapper. The ship-set compiler emits one
// `defineComponent('my-tag', factory)` call per .set file. The factory
// is the user's script + binding wiring; defineComponent registers
// the wrapper class with `customElements.define`.
//
// See .artifacts/specs/ship-set/03-runtime-architecture.md § Component wrapper.

import type { Disposer } from './bind'
import type { Signal } from './signal'
import { createScope, withScope, type EffectScope } from './effect-scope'
import { attachHostApi } from './host-api'

// The declarative `component()` factory lives in component-factory.ts to
// keep this file under the 300-line cap. Re-exported here so the public
// API can be imported from a single module.
export {
  component,
  prop,
  type PropDef,
  type ComponentDef,
  type ComponentSetupContext,
} from './component-factory'

/**
 * The factory the compiler emits per component. Receives the host
 * element, a place to attach prop signals, and (for form-associated
 * components) the ElementInternals instance attached at construction
 * time. Returns a cleanup function that runs on disconnect.
 */
export type ComponentFactory = (
  host: HTMLElement,
  propSignals: Record<string, Signal<unknown>>,
  internals?: ElementInternals,
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
  /**
   * When true, the emitted custom element declares
   * `static formAssociated = true` and calls `attachInternals()` at
   * construction. The factory receives the ElementInternals as its
   * third argument so script-block code can call setFormValue,
   * setValidity, checkValidity, and friends.
   */
  formAssociated?: boolean
}

/**
 * Register a ship-set component as a custom element. The wrapper class
 * observes the schema's attributes, decodes incoming changes into prop
 * signals, runs `factory` on connect, and runs the returned cleanup on
 * disconnect.
 *
 * Idempotent — calling twice with the same tag is a no-op (the browser
 * would otherwise throw on the second `customElements.define`). A
 * `console.warn` fires in dev so the duplicate doesn't go unnoticed;
 * silenced when the registered class is the very same factory reference
 * (re-import from HMR).
 *
 * The ship-set compiler emits exactly one call per `.set` file. Authors
 * shouldn't need to reach for this directly.
 *
 * @param tag Custom element tag name (must contain a hyphen).
 * @param schema Observed-attribute list + per-attribute decoder map +
 *   optional `formAssociated` flag.
 * @param factory Per-instance setup; receives the host, prop-signal map,
 *   and (when form-associated) ElementInternals. Returns a disposer
 *   that runs on disconnect.
 *
 * @example
 * ```ts
 * defineComponent('my-counter', {
 *   observedAttrs: ['initial'],
 *   props: { initial: { camel: 'initial', decode: decodeNumber } },
 * }, (host, props) => {
 *   const count = props.initial as Signal<number>
 *   return () => {} // cleanup
 * })
 * ```
 *
 * @see camelOf for attribute → prop name translation.
 */
const warnedDuplicateTags = new Set<string>()

export function defineComponent(
  tag: string,
  schema: ComponentSchema,
  factory: ComponentFactory,
): void {
  if (customElements.get(tag)) {
    // Silent overwrite would mask bugs (two modules trying to register
    // the same tag). Warn once per tag. HMR cycles legitimately re-run
    // the registration with the SAME factory; that case stays silent
    // because we never reach this branch — the first define captured
    // the factory and the second call is a no-op for the new factory.
    if (!warnedDuplicateTags.has(tag)) {
      warnedDuplicateTags.add(tag)
      console.warn(
        `[sui-runtime] customElements.define("${tag}", …) called twice. ` +
          `The first registration wins; this call is a no-op. Likely cause: ` +
          `two modules independently registering the same tag.`,
      )
    }
    return
  }

  // Two class shapes — form-associated requires `static formAssociated`
  // to be present at class-definition time so the browser registers
  // ElementInternals support. The non-associated path skips the
  // constructor + internals.
  if (schema.formAssociated) {
    class ShipSetFormElement extends HTMLElement {
      static formAssociated = true
      static get observedAttributes(): string[] {
        return schema.observedAttrs
      }

      private cleanup: Disposer | undefined
      private scope: EffectScope | undefined
      private propSignals: Record<string, Signal<unknown>> = {}
      private internals: ElementInternals

      constructor() {
        super()
        this.internals = this.attachInternals()
      }

      connectedCallback() {
        this.dataset.component = tag
        attachHostApi(this)
        this.scope = createScope()
        this.cleanup = withScope(this.scope, () =>
          factory(this, this.propSignals, this.internals),
        )
      }

      disconnectedCallback() {
        try {
          this.cleanup?.()
        } catch (e) {
          console.error(`[ship-set:${tag}] cleanup threw:`, e)
        }
        this.cleanup = undefined
        try {
          this.scope?.dispose()
        } catch (e) {
          console.error(`[ship-set:${tag}] scope dispose threw:`, e)
        }
        this.scope = undefined
      }

      attributeChangedCallback(attrName: string, _old: string, value: string) {
        const meta = schema.props[attrName]
        if (!meta) return
        const sig = this.propSignals[meta.camel]
        if (sig) sig.set(meta.decode(value))
      }

      // Form-association lifecycle hooks. The runtime exposes them via
      // user-supplied `internals.*` calls in mount/effect blocks; the
      // browser also dispatches these on the host so authors who want
      // to override (e.g. for restore) can attach listeners directly.
      formResetCallback() {
        this.dispatchEvent(new CustomEvent('ship-set:form-reset'))
      }
      formStateRestoreCallback(state: string | null, mode: string) {
        this.dispatchEvent(
          new CustomEvent('ship-set:form-restore', { detail: { state, mode } }),
        )
      }
      formDisabledCallback(disabled: boolean) {
        this.dispatchEvent(
          new CustomEvent('ship-set:form-disabled', { detail: { disabled } }),
        )
      }
    }
    customElements.define(tag, ShipSetFormElement)
    return
  }

  class ShipSetElement extends HTMLElement {
    static get observedAttributes(): string[] {
      return schema.observedAttrs
    }

    private cleanup: Disposer | undefined
    private scope: EffectScope | undefined
    private propSignals: Record<string, Signal<unknown>> = {}

    connectedCallback() {
      this.dataset.component = tag
      attachHostApi(this)
      this.scope = createScope()
      this.cleanup = withScope(this.scope, () =>
        factory(this, this.propSignals),
      )
    }

    disconnectedCallback() {
      try {
        this.cleanup?.()
      } catch (e) {
        console.error(`[ship-set:${tag}] cleanup threw:`, e)
      }
      this.cleanup = undefined
      try {
        this.scope?.dispose()
      } catch (e) {
        console.error(`[ship-set:${tag}] scope dispose threw:`, e)
      }
      this.scope = undefined
    }

    attributeChangedCallback(attrName: string, _old: string, value: string) {
      const meta = schema.props[attrName]
      if (!meta) return
      const sig = this.propSignals[meta.camel]
      if (sig) sig.set(meta.decode(value))
    }
  }

  customElements.define(tag, ShipSetElement)
}

/**
 * Convert a kebab-case HTML attribute name to its camelCase prop name.
 * Single-word attributes pass through unchanged.
 *
 * @param attr Attribute name, lowercased and possibly hyphenated.
 * @returns Camel-cased identifier suitable for a JS property name.
 *
 * @example
 * ```ts
 * camelOf('fetch-url') // 'fetchUrl'
 * camelOf('min-chars') // 'minChars'
 * camelOf('count')     // 'count'
 * ```
 *
 * @see defineComponent — schema entries map kebab → camel via this rule.
 */
export function camelOf(attr: string): string {
  return attr.replace(/-([a-z])/g, (_, c: string) => c.toUpperCase())
}
