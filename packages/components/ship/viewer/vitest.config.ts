import { pathToFileURL } from 'node:url'
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'
import type { Plugin } from 'vitest/config'

// happy-dom (not jsdom): the dispatcher composes `@ship/sui-primitives`
// (sui-markdown, sui-code, sui-json-view, sui-empty-state) which inject modern
// scoped CSS; jsdom 25's parser throws "Could not parse CSS stylesheet" on
// mount, happy-dom tolerates it. The default include globs
// (`test/**/*.test.ts` + `src/**/*.test.ts`) pick up the relocated dispatcher
// suite (under test/) and the co-located engine spec (under src/engine/).
const base = suiComponentVitestConfig({ environment: 'happy-dom' })

// The dispatcher spec locates the `.sui` source with the canonical
// `readFileSync(new URL('../src/ship-viewer/ship-viewer.sui', import.meta.url))`
// idiom. Vite's `assetImportMetaUrl` plugin rewrites `new URL('<lit>',
// import.meta.url)` into `new URL('<lit>', self.location)`; under happy-dom
// `self.location` is `http://localhost:3000/`, so the read resolves to an http
// URL and `readFileSync` throws "The URL must be of scheme file". Pin
// `import.meta.url` in the spec files to the literal `file://` URL of the test
// file *before* the asset plugin runs (`enforce: 'pre'`), so the base is a real
// file URL the WHATWG `URL` resolves correctly and the asset rewrite no longer
// matches.
const pinImportMetaUrl: Plugin = {
  name: 'ship-viewer:pin-import-meta-url',
  enforce: 'pre',
  transform(code: string, id: string) {
    const path = id.split('?', 1)[0] ?? id
    if (!/[\\/]test[\\/][^\\/]+\.test\.ts$/.test(path)) return null
    if (!code.includes('import.meta.url')) return null
    const literal = JSON.stringify(pathToFileURL(path).href)
    return { code: code.split('import.meta.url').join(literal), map: null }
  },
}

// `renderComponent` compiles the dispatcher `.sui` from source via the WASM
// helper (`@ship/sui-dom/test-utils` → `compileSui`), whose opts JSON omits the
// `shadow` flag. The WASM compiler defaults `shadow:false`, so an inline-compiled
// dispatcher would mount to light DOM — but it renders into a shadow root in
// production (the bin/FFI compiler that produces the imported `.compiled.js`
// primitives hardcodes `shadow:true`; the `.sui` declares `:host` + `part=` +
// slots accordingly). The spec asserts on `el.shadowRoot`. Inject `shadow:true`
// into the component-mode opts JSON so the inline test compile matches the
// production shadow-DOM shape (component mode only — view compiles, which pass
// no `componentName`, are untouched).
const shadowForComponentCompile: Plugin = {
  name: 'ship-viewer:shadow-for-component-compile',
  enforce: 'pre',
  transform(code: string, id: string) {
    if (!/[\\/]test-utils[\\/]compile-sui\.ts$/.test(id)) return null
    const needle = 'setupUrl: opts.setupMarker,'
    if (!code.includes(needle)) return null
    return {
      code: code.replace(needle, `${needle}\n    shadow: opts.componentName != null,`),
      map: null,
    }
  },
}

export default {
  ...base,
  plugins: [pinImportMetaUrl, shadowForComponentCompile, ...(base.plugins ?? [])],
}
