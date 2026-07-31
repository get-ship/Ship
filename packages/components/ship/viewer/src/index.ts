// @ship/viewer — the content-renderer ROUTER pack (`<ship-viewer>` dispatcher).
//
// The dispatcher's <tag>.compiled.js calls customElements.define() on import, so
// a single `import '@ship/viewer'` registers the `ship-viewer` tag globally.
//
// `<ship-viewer>` reads a resolved source and routes it to the right renderer
// through @ship/sail-media's pure createMimeRegistry() engine, seeded with this
// router's default kind→TAG table (`engine/registry.ts` — the dispatch
// decision). The leaf renderers it dispatches to are the @ship/sail-media
// `sui-media-*` tags; the markdown / code / json / diff / tabular arms compose
// @ship/sail-primitives targets; the pdf arm renders `<ship-pdf>` from @ship/pdf.
// All are declared dependencies — importing this barrel registers the pdf tag
// (`@ship/pdf` self-registers on import) alongside the host's own
// `import '@ship/sail-media'` + `import '@ship/sail-primitives'`.
import '@ship/pdf'
import './ship-viewer/ship-viewer.compiled.js'

// The router's default kind→TAG table — exported so a host can extend it
// (`createMimeRegistry({ entries: [...custom, ...defaultEntries()] })`).
export { defaultEntries } from './engine/registry'
