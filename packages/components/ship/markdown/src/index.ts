// @ship/markdown — Ship Markdown pack (ship-* tags).
//
// Each <tag>.compiled.js calls customElements.define() on import, so a
// single `import '@ship/markdown'` registers every tag globally.
//
// <ship-markdown-editor> is a form-associated TipTap v3 markdown editor;
// it composes `<sui-icon>` from `@ship/sui-primitives` (a declared dep)
// for its toolbar glyphs, so the host must register that pack too.
import './ship-markdown-editor/ship-markdown-editor.compiled.js'
