# Vendored pdf.js (`pdfjs-dist@6.0.227`)

Self-contained ESM build of [pdf.js](https://github.com/mozilla/pdf.js),
vendored 2026-06-02. **Not** an npm dependency — `@ship/pdf` wraps this engine
(SHIP-tier vendored engine per `docs/ship-adrs/.../ship-component-architecture.md`).

| File           | What it is                                              |
| -------------- | ------------------------------------------------------- |
| `pdfjs.mjs`    | Main thread API (`getDocument`, proxies, `GlobalWorkerOptions`). |
| `pdf.worker.mjs` | The PDF parsing/rendering worker pdf.js offloads to.  |
| `pdfjs.d.mts`  | Hand-maintained types, faithful to the build's exports. |
| `LICENSE`      | Apache-2.0, verbatim from upstream.                     |

Both `.mjs` files are self-contained: no bare/external imports, no runtime npm
deps. (`pdfjs.mjs` has one `createRequire("@napi-rs/canvas")` inside
`NodeCanvasFactory` — a Node-only branch never hit in a browser/webview, where
`DOMCanvasFactory` is used instead.)

## Pointing pdf.js at the vendored worker — offline, no CDN

pdf.js spawns a Web Worker for parsing. By default it derives the worker URL
from a CDN/`workerSrc` guess, which would hit the network. To stay offline and
in-product, give it the **vendored** worker explicitly. Two equivalent ways:

### 1. `GlobalWorkerOptions.workerSrc` — a URL the host already serves

```ts
import { GlobalWorkerOptions, getDocument } from './vendor/pdfjs/pdfjs.mjs'

// Resolve the vendored worker relative to this module (bundler rewrites it to
// the emitted asset URL — same-origin, no CDN). Set once at startup.
GlobalWorkerOptions.workerSrc = new URL(
  './vendor/pdfjs/pdf.worker.mjs',
  import.meta.url,
).href

const doc = await getDocument(src).promise
```

`import.meta.url` keeps the URL same-origin (the host that served the component),
so the worker loads from the product, never a CDN.

### 2. `workerPort` — hand pdf.js a Worker you constructed

When you want full control over worker construction (e.g. a confined/SES host,
or to reuse one worker across documents):

```ts
import { GlobalWorkerOptions, getDocument } from './vendor/pdfjs/pdfjs.mjs'

const worker = new Worker(
  new URL('./vendor/pdfjs/pdf.worker.mjs', import.meta.url),
  { type: 'module' },
)
GlobalWorkerOptions.workerPort = worker
// or per-document: getDocument({ ...src, worker: PDFWorker.fromPort({ port: worker }) })
```

Either path resolves the worker to the vendored file via `import.meta.url`, so
it works offline / in-product with no network fetch. Prefer `workerSrc` for the
simple case; use `workerPort` when the worker lifecycle must be owned explicitly.

## Re-vendoring

```sh
npm pack pdfjs-dist@<version>
tar xzf pdfjs-dist-<version>.tgz
cp package/build/pdf.mjs        src/vendor/pdfjs/pdfjs.mjs
cp package/build/pdf.worker.mjs src/vendor/pdfjs/pdf.worker.mjs
# refresh LICENSE header (version + date) and re-check the d.mts export surface
```
