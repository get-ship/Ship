// Component-registration helper.
//
// Each ported primitive's `.ts` file imports its `.html.set` template
// (esbuild's `text` loader on `.set` returns the file contents as a
// string) and optionally a sibling `.css` (also text-loaded). The
// helper splices the CSS into the template's `<component>` block as a
// `<style>` child and feeds the result to `hydrate()`, which parses
// the directive and registers the custom element.
//
// Why splice CSS in at registration time instead of authoring inline
// `<style>` blocks in every `.html.set`?
//
//   - A primitive's CSS surface routinely runs 100–500 lines (tone
//     matrices, focus rings, motion). Authoring it inline pushes the
//     `.html.set` past the 300-line cap on every nontrivial port.
//   - Editors handle stand-alone `.css` natively (linting, formatting,
//     completion). Editing inside an HTML `<style>` element loses
//     that.
//   - The .ts file is the integration point already — `.html.set` is
//     declarative, .ts is imperative; co-locating the imports keeps
//     the integration boundary in one file.
//
// Hydrate is invoked with a throwaway container — for a `<component>`
// directive, hydrate registers the custom element via the runtime's
// `defineComponent` and returns a no-op disposer; nothing renders into
// the container. Subsequent `<set-*>` instances appear when authors
// place the tag in a parent template.

import { hydrate } from '@ship-set/dom'

/**
 * Options accepted by `defineComponent`. Both fields are optional; the
 * minimal call is `defineComponent(template)` for components with
 * inline `<style>` already in the template.
 */
export interface DefineComponentOptions {
  /**
   * CSS to attach inside the component's shadow root. Authors usually
   * pass an `import css from './tag.css'` (esbuild text loader). The
   * helper splices `<style>${css}</style>` immediately before the
   * `</component>` closing tag of the template.
   *
   * For light-DOM components, ignore this and inline whatever scoping
   * the framework's auto-prefix provides.
   */
  styles?: string

  /**
   * Document to register against (defaults to the page document). Pass
   * a jsdom Document in tests; the helper otherwise reads
   * `globalThis.document`.
   */
  doc?: Document
}

/**
 * Register a `.html.set` ported primitive. Splices `opts.styles` (when
 * provided) into the template's `<component>` block, then calls
 * `hydrate()` to parse the directive and define the custom element.
 *
 * Idempotent — the runtime's `defineComponent` skips re-registration
 * if the tag is already in the registry.
 *
 * @param template Raw `.html.set` source.
 * @param opts Optional CSS + document override.
 */
export function defineComponent(
  template: string,
  opts: DefineComponentOptions = {},
): void {
  const doc = opts.doc ?? (typeof document !== 'undefined' ? document : undefined)
  if (!doc) {
    throw new Error(
      '[ship-set/components] defineComponent: no document available — ' +
        'pass `opts.doc` when calling outside the browser/jsdom.',
    )
  }
  const merged = opts.styles
    ? template.replace('</component>', `<style>${opts.styles}</style></component>`)
    : template
  const container = doc.createElement('div')
  hydrate(merged, undefined, { container })
}
