// @ship/code — Ship Code pack (editable code surface backed by vendored CodeMirror 6).
//
// The `<tag>.compiled.js` import calls customElements.define() on load, so a
// single `import '@ship/code'` registers the tag globally (same pattern as
// @ship/agents). Slice 2 ships <ship-code-editor>; the diff surface (Slice 3,
// <ship-code-diff>) lands as a follow-up.
import './ship-code-editor/ship-code-editor.compiled.js'

export { createCodeEditor } from './engine/create-code-editor.js'
export { resolveLanguage, languageIds } from './engine/languages.js'
export { codeEditorTheme } from './engine/theme.js'
export type { CodeEditor, CodeEditorOptions } from './engine/types.js'
