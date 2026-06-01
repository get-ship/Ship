import { pathToFileURL } from 'node:url'
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'
import type { Plugin } from 'vitest/config'

// happy-dom (not jsdom): the renderers compose `@ship/sui-primitives`
// (sui-table, sui-empty-state, sui-markdown, sui-code) which inject modern
// scoped CSS; jsdom 25's parser throws "Could not parse CSS stylesheet" on
// mount, happy-dom tolerates it. The default include globs
// (`test/**/*.test.ts` + `src/**/*.test.ts`) pick up both the relocated suites
// (under test/) and the co-located engine specs (under src/engine/).
const base = suiComponentVitestConfig({ environment: 'happy-dom' })

// The spec suites locate a renderer's `.sui` source with the canonical
// `readFileSync(new URL('../src/<tag>/<tag>.sui', import.meta.url))` idiom.
// Vite's `assetImportMetaUrl` plugin rewrites `new URL('<lit>', import.meta.url)`
// into `new URL('<lit>', self.location)`; under happy-dom `self.location` is
// `http://localhost:3000/`, so the read resolves to an http URL and
// `readFileSync` throws "The URL must be of scheme file". Pin `import.meta.url`
// in the spec files to the literal `file://` URL of the test file *before* the
// asset plugin runs (`enforce: 'pre'`), so the base is a real file URL the WHATWG
// `URL` resolves correctly and the asset rewrite no longer matches.
const pinImportMetaUrl: Plugin = {
  name: 'sui-viewers:pin-import-meta-url',
  enforce: 'pre',
  transform(code: string, id: string) {
    const path = id.split('?', 1)[0] ?? id
    if (!/[\\/]test[\\/][^\\/]+\.test\.ts$/.test(path)) return null
    if (!code.includes('import.meta.url')) return null
    const literal = JSON.stringify(pathToFileURL(path).href)
    return { code: code.split('import.meta.url').join(literal), map: null }
  },
}

// `renderComponent` compiles a renderer `.sui` from source via the WASM helper
// (`@ship/sui-dom/test-utils` → `compileSui`), whose opts JSON omits the `shadow`
// flag. The WASM compiler defaults `shadow:false`, so an inline-compiled renderer
// would mount to light DOM — but every viewer renders into a shadow root in
// production (the bin/FFI compiler that produces the imported `.compiled.js`
// primitives hardcodes `shadow:true`; the `.sui` files declare `:host` + `part=`
// + slots accordingly). The spec suites assert on `el.shadowRoot`. Inject
// `shadow:true` into the component-mode opts JSON so the inline test compile
// matches the production shadow-DOM shape (component mode only — view compiles,
// which pass no `componentName`, are untouched).
const shadowForComponentCompile: Plugin = {
  name: 'sui-viewers:shadow-for-component-compile',
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
