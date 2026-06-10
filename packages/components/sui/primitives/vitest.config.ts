import { pathToFileURL } from 'node:url'
import { suiComponentVitestConfig } from '@ship/test-setup/vitest'
import type { Plugin } from 'vitest/config'

// Default env (jsdom) for the whole suite — every existing component test
// relies on it (the form components crash under happy-dom's HTMLParser). The one
// renderComponent suite that needs happy-dom (sui-calendar, whose scoped CSS
// trips jsdom's parser) opts in PER-FILE via a `// @vitest-environment happy-dom`
// docblock, so it can never regress the rest of the suite again.
const base = suiComponentVitestConfig()

// Pin `import.meta.url` in spec files to the literal `file://` URL of the test
// file BEFORE vite's asset plugin rewrites `new URL('<lit>', import.meta.url)`
// to `self.location` (an http URL under happy-dom, which `readFileSync` rejects).
// Mirrors the sui-viewers config — needed now that primitives ships its first
// renderComponent suite (sui-calendar). Scoped to `*.test.ts` files.
const pinImportMetaUrl: Plugin = {
  name: 'sui-primitives:pin-import-meta-url',
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
// compile matches the production shadow-DOM shape (the bin/FFI compiler that
// produces `.compiled.js` hardcodes shadow:true; the inline WASM/native default
// is shadow:false). Component mode only — view compiles are untouched.
const shadowForComponentCompile: Plugin = {
  name: 'sui-primitives:shadow-for-component-compile',
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
