// @ship/viewer — the content-renderer ROUTER pack (`<ship-viewer>` dispatcher).
//
// The dispatcher's <tag>.compiled.js calls customElements.define() on import, so
// a single `import '@ship/viewer'` registers the `ship-viewer` tag globally.
//
// `<ship-viewer>` reads a resolved source and routes it to the right renderer
// through @ship/sui-media's pure createMimeRegistry() engine, seeded with this
// router's default kind→TAG table (`engine/registry.ts` — the dispatch
// decision). The leaf renderers it dispatches to are the @ship/sui-media
// `sui-media-*` tags; the markdown / code / json / diff / tabular arms compose
// @ship/sui-primitives targets. Both are declared dependencies the host
// registers via `import '@ship/sui-media'` + `import '@ship/sui-primitives'`.
import './ship-viewer/ship-viewer.compiled.js'

// The router's default kind→TAG table — exported so a host can extend it
// (`createMimeRegistry({ entries: [...custom, ...defaultEntries()] })`).
export { defaultEntries } from './engine/registry'
