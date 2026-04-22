/**
 * Module declarations for raw-text asset imports. esbuild's `--loader:.svg=text`
 * flag (set in `bundle:elements` package.json script) returns SVG file
 * contents as a string, but TypeScript needs this hint to type-check the
 * imports.
 */
declare module '*.svg' {
  const content: string
  export default content
}
