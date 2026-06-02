import { pathToFileURL } from 'node:url'
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'
import type { Plugin } from 'vitest/config'

// Base is the shared component config (jsdom default). The <ship-code-editor>
// renderComponent suite opts into happy-dom PER-FILE (CodeMirror injects modern
// scoped CSS that jsdom 25's parser rejects — the same reason sui-calendar pins
// happy-dom). That brings the same two plugin needs as the primitives pack:
const base = suiComponentVitestConfig()

// Pin `import.meta.url` in spec files to the literal `file://` URL of the test
// file BEFORE vite's asset plugin rewrites `new URL('<lit>', import.meta.url)`
// to `self.location` (an http URL under happy-dom, which `readFileSync` rejects).
// Mirrors the sui-primitives config. Scoped to `*.test.ts` files.
const pinImportMetaUrl: Plugin = {
  name: 'ship-code:pin-import-meta-url',
  enforce: 'pre',
  transform(code: string, id: string) {
    const path = id.split('?', 1)[0] ?? id
    if (!/\.test\.ts$/.test(path)) return null
    if (!code.includes('import.meta.url')) return null
    const literal = JSON.stringify(pathToFileURL(path).href)
    return { code: code.split('import.meta.url').join(literal), map: null }
  },
}

// Inject `shadow:true` into the component-mode compile opts so an inline test
// compile matches the production shadow-DOM shape (the FFI compiler that emits
// `.compiled.js` hardcodes shadow:true; the inline native default is
// shadow:false). Component mode only — view compiles are untouched.
const shadowForComponentCompile: Plugin = {
  name: 'ship-code:shadow-for-component-compile',
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
