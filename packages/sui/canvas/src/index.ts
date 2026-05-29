// @sui/canvas — the drawing-surface pack.
//
// The compiled <sui-canvas> tag calls customElements.define() on import,
// so a single `import '@sui/canvas'` registers the tag globally.
//
// Bundle note: importing this pulls in @excalidraw/excalidraw + a React 19
// tree (~1 MB minified). Import only where you actually mount a canvas.
import './sui-canvas/sui-canvas.compiled.js'
