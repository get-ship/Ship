// Tell TypeScript that `import x from './foo.html.set'` is a string —
// esbuild (production bundles) and vitest (tests) both load `.html.set`
// files as raw text.
declare module '*.html.set' {
  const source: string
  export default source
}

// `<tag>.compiled.js` files are emitted by `ship-set-compiler` at
// build time (gitignored). Components import them for side effects:
// the compiled module's top-level `component(...)` call registers
// the custom element. The shape is "no exports, no contract" — only
// the side effect matters.
declare module '*.compiled.js'
