// Vitest setup for `@ship/markdown` (runs under happy-dom — see the per-file
// `// @vitest-environment happy-dom` docblock in the spec). Installs the shared
// Web-API shims from `@ship/test-setup` (ResizeObserver, matchMedia, Popover,
// Dialog, ToggleEvent, PointerEvent) before any specimen mounts; <ship-markdown-editor>
// composes `<sui-icon>` from `@ship/sui-primitives`, which probes those APIs.

import { initJsdomPolyfills } from '@ship/test-setup'

initJsdomPolyfills()

// happy-dom does not implement `attachInternals`, which form-associated custom
// elements call in their constructor — <ship-markdown-editor> is form-associated
// (it carries a form value). Stub it so the element connects; the tests assert
// DOM/ARIA/event flow, not form-internals behaviour, so a no-op ElementInternals
// is sufficient. Mirrors the @ship/sui-media setup precedent.
const proto = HTMLElement.prototype as HTMLElement & { attachInternals?: () => unknown }
if (typeof proto.attachInternals !== 'function') {
  proto.attachInternals = function attachInternals(): unknown {
    return {
      setFormValue() {},
      setValidity() {},
      checkValidity() {
        return true
      },
      reportValidity() {
        return true
      },
      get form() {
        return null
      },
      get labels() {
        return [] as unknown as NodeList
      },
      get shadowRoot() {
        return (this as unknown as { shadowRoot: ShadowRoot | null }).shadowRoot ?? null
      },
    }
  }
}
