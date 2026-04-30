// Component-registration helper — DEPRECATED stub kept during the
// Arc B-1 transition window.
//
// Until Arc B-1, this helper read the `.html.set` template, optionally
// spliced a sibling `.css` into the `<component>` block, and called
// `hydrate()` to parse the directive and register the custom element.
// Registration now happens ahead-of-time: `ship-set-compiler` emits a
// `<tag>.compiled.js` per primitive, the .ts file side-effect imports
// it, and the compiled module's top-level `component(...)` call
// registers the element. No runtime parser is involved.
//
// The function below is preserved as a no-op so any straggler caller
// surfaces a console warning instead of a hard failure. Plan: remove
// this file in the Arc B-2 follow-up once the deprecation window
// closes (one week of zero callers).

/**
 * Options once accepted by `defineComponent`. Both fields are now
 * ignored; CSS is sourced from the `.html.set` `<style>` block (or
 * the `--styles <css>` flag passed to `ship-set-compiler`), and the
 * document is always the page document at registration time.
 */
export interface DefineComponentOptions {
  /** @deprecated Inline styles in the `.html.set` `<style>` child of
   *  `<component>` instead, or pass them to `ship-set-compiler --styles`. */
  styles?: string
  /** @deprecated Components register against the page document at
   *  module-import time; no document override is supported. */
  doc?: Document
}

/**
 * @deprecated Components are AOT-compiled by `ship-set-compiler`. This
 * helper exists only as a no-op stub during the migration window;
 * call sites will be removed in the follow-up Arc B-2. Do not add new
 * callers — author components as `.html.set` + side-effect-importing
 * `.ts` files instead, and register implicitly via
 * `import './<tag>/<tag>'`.
 */
export function defineComponent(
  _template: string,
  _opts: DefineComponentOptions = {},
): void {
  if (typeof console !== 'undefined') {
    console.warn(
      '[ship-set/components] defineComponent is deprecated; ' +
        'components are now AOT-compiled by ship-set-compiler.',
    )
  }
}
