// Tell TypeScript that `import x from './foo.html.set'` is a string —
// esbuild (production bundles) and vitest (tests) both load `.html.set`
// files as raw text.
declare module '*.html.set' {
  const source: string
  export default source
}
