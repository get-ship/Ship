// Design-token theme for the editor. NO raw hex/rgb — every color slot maps to
// a `var(--token)` from the Ship design system. Light/dark is selected by the
// caller (the `dark` flag flips CM's internal dark heuristics; the actual
// colors come from whichever token values the active theme resolves to).
import { EditorView, HighlightStyle, syntaxHighlighting, tags } from '../vendor/codemirror/codemirror.mjs'
import type { Extension } from '../vendor/codemirror/codemirror.mjs'

/** EditorView.theme mapping CM structural slots to design tokens. */
function viewTheme(dark: boolean): Extension {
  return EditorView.theme(
    {
      '&': {
        color: 'var(--foreground)',
        backgroundColor: 'var(--background)',
        fontFamily: 'var(--font-mono)',
      },
      '.cm-content': {
        caretColor: 'var(--foreground)',
        fontFamily: 'var(--font-mono)',
      },
      '.cm-cursor, .cm-dropCursor': { borderLeftColor: 'var(--foreground)' },
      '&.cm-focused': { outline: 'none' },
      '&.cm-focused .cm-selectionBackground, .cm-selectionBackground, .cm-content ::selection': {
        backgroundColor: 'var(--accent)',
      },
      '.cm-gutters': {
        backgroundColor: 'var(--background)',
        color: 'var(--muted-foreground)',
        borderRight: '1px solid var(--border)',
      },
      '.cm-activeLine': { backgroundColor: 'var(--muted)' },
      '.cm-activeLineGutter': { backgroundColor: 'var(--muted)' },
      '.cm-lineNumbers .cm-gutterElement': { color: 'var(--muted-foreground)' },
      '.cm-selectionMatch': { backgroundColor: 'var(--accent)' },
      '&.cm-focused .cm-matchingBracket': {
        outline: '1px solid var(--ring)',
        backgroundColor: 'transparent',
      },
      '.cm-tooltip': {
        backgroundColor: 'var(--background)',
        border: '1px solid var(--border)',
        color: 'var(--foreground)',
      },
      '.cm-panels': {
        backgroundColor: 'var(--background)',
        color: 'var(--foreground)',
      },
    },
    { dark },
  )
}

/** HighlightStyle mapping lezer tags to the --syntax-* token set. */
const highlight = HighlightStyle.define([
  { tag: tags.keyword, color: 'var(--syntax-keyword)' },
  { tag: tags.controlKeyword, color: 'var(--syntax-keyword)' },
  { tag: tags.operatorKeyword, color: 'var(--syntax-keyword)' },
  { tag: tags.string, color: 'var(--syntax-string)' },
  { tag: tags.special(tags.string), color: 'var(--syntax-string)' },
  { tag: tags.comment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.lineComment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.blockComment, color: 'var(--syntax-comment)', fontStyle: 'italic' },
  { tag: tags.number, color: 'var(--syntax-number)' },
  { tag: tags.bool, color: 'var(--syntax-number)' },
  { tag: [tags.function(tags.variableName), tags.function(tags.propertyName)], color: 'var(--syntax-function)' },
  { tag: tags.typeName, color: 'var(--syntax-type)' },
  { tag: tags.className, color: 'var(--syntax-type)' },
  { tag: [tags.variableName, tags.propertyName], color: 'var(--syntax-variable)' },
  { tag: tags.tagName, color: 'var(--syntax-tag)' },
  { tag: tags.angleBracket, color: 'var(--syntax-tag)' },
  { tag: [tags.attributeName, tags.attributeValue], color: 'var(--syntax-attr)' },
])

/** Token-driven editor theme: structural slots + syntax highlighting. */
export function codeEditorTheme(dark: boolean): Extension {
  return [viewTheme(dark), syntaxHighlighting(highlight)]
}
