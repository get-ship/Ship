// @ship/sui-viewer — the shared component-viewing components.
//
// THE key unlock: one set of reusable viewing primitives every surface
// (studio, sui-devtools, eval, any L3 app) composes — instead of N bespoke
// viewer implementations. The pack mints (across Slices 1-5):
//   - sui-render          — the live mount engine (view-mode + component-mode)
//   - sui-prop-knobs       — the CEM-driven knob playground
//   - sui-prop-matrix      — the Storybook-style state sweep
//   - sui-source-panel     — source + paste-able snippet + the one copy path
//   - sui-component-viewer — the composition primitive (the embeddable tag)
//
// Each `<tag>.compiled.js` calls `customElements.define()` on import, so as the
// tags land they are side-effect-registered here (the charts-pack convention).
// The `.compiled.js` is emitted by the deck build (not committed) — the import
// is resolved at bundle time, mirroring `@ship/sui-charts`/`@ship/sui-primitives`.

// Slice 1 — the engine primitive.
import './sui-render/sui-render.compiled.js'

// WASM-in-browser bootstrap (Slice 0). Framework-pure: takes the wasm URL as a
// param, exposes the bound `compile()` the host injects into the primitives via
// `prop:compile={fn}`. This is the only host-coupled mechanism in the pack, and
// it is import-free of any daemon/SDK surface.
export {
  ensureWasm,
  isWasmReady,
  makeCompile,
  type CompileFn,
  type WasmCompileOpts,
  type WasmCompileResult,
} from './wasm-bootstrap'
