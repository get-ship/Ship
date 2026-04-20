/**
 * CodeMirror theme using Ship design tokens.
 *
 * Uses CSS custom properties from @ship/set/tokens.css so the theme
 * automatically adapts to light/dark mode via the `.dark` class on
 * documentElement. All color references resolve at paint time.
 */

import { EditorView } from '@codemirror/view'
import { HighlightStyle, syntaxHighlighting } from '@codemirror/language'
import { tags } from '@lezer/highlight'

export const shipEditorTheme = EditorView.theme({
  '&': {
    backgroundColor: 'var(--card)',
    color: 'var(--foreground)',
    fontSize: 'inherit',
    fontFamily: 'inherit',
  },
  '.cm-content': {
    caretColor: 'var(--foreground)',
    fontFamily: 'inherit',
    padding: '8px 0',
  },
  '.cm-cursor, .cm-dropCursor': {
    borderLeftColor: 'var(--foreground)',
    borderLeftWidth: '2px',
  },
  '&.cm-focused .cm-selectionBackground, .cm-selectionBackground': {
    background: 'color-mix(in oklch, var(--primary) 20%, transparent)',
  },
  '.cm-activeLine': {
    backgroundColor: 'color-mix(in oklch, var(--muted) 30%, transparent)',
  },
  '.cm-gutters': {
    backgroundColor: 'var(--muted)',
    color: 'var(--muted-foreground)',
    border: 'none',
    borderRight: '1px solid var(--border)',
  },
  '.cm-activeLineGutter': {
    backgroundColor: 'color-mix(in oklch, var(--muted) 60%, transparent)',
    color: 'var(--foreground)',
  },
  '.cm-foldPlaceholder': {
    backgroundColor: 'var(--muted)',
    border: '1px solid var(--border)',
    color: 'var(--muted-foreground)',
  },
  '.cm-tooltip': {
    backgroundColor: 'var(--popover)',
    color: 'var(--popover-foreground)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius-md)',
    boxShadow: 'var(--shadow-md)',
  },
  '.cm-tooltip-autocomplete': {
    '& > ul > li[aria-selected]': {
      backgroundColor: 'var(--muted)',
    },
  },
  '.cm-searchMatch': {
    backgroundColor: 'color-mix(in oklch, var(--info) 25%, transparent)',
  },
  '.cm-searchMatch.cm-searchMatch-selected': {
    backgroundColor: 'color-mix(in oklch, var(--primary) 30%, transparent)',
  },
  '.cm-placeholder': {
    color: 'var(--muted-foreground)',
    fontStyle: 'italic',
  },
  '&.cm-focused': {
    outline: 'none',
  },
  '.cm-scroller': {
    fontFamily: 'inherit',
    lineHeight: 'inherit',
  },
})

const shipHighlightStyle = HighlightStyle.define([
  { tag: tags.keyword, color: 'var(--primary)' },
  { tag: tags.operator, color: 'var(--primary)' },
  { tag: tags.controlKeyword, color: 'var(--primary)', fontWeight: '500' },
  { tag: tags.definitionKeyword, color: 'var(--primary)' },
  { tag: tags.moduleKeyword, color: 'var(--primary)' },

  { tag: tags.string, color: 'var(--success)' },
  { tag: tags.regexp, color: 'var(--success)' },
  { tag: tags.special(tags.string), color: 'var(--success)' },

  { tag: tags.number, color: 'var(--info)' },
  { tag: tags.bool, color: 'var(--info)' },
  { tag: tags.null, color: 'var(--info)' },

  { tag: tags.comment, color: 'var(--muted-foreground)', fontStyle: 'italic' },
  { tag: tags.blockComment, color: 'var(--muted-foreground)', fontStyle: 'italic' },
  { tag: tags.lineComment, color: 'var(--muted-foreground)', fontStyle: 'italic' },

  { tag: tags.variableName, color: 'var(--foreground)' },
  { tag: tags.definition(tags.variableName), color: 'var(--foreground)', fontWeight: '500' },
  { tag: tags.function(tags.variableName), color: 'var(--accent)' },

  { tag: tags.typeName, color: 'var(--accent)' },
  { tag: tags.className, color: 'var(--accent)' },
  { tag: tags.propertyName, color: 'var(--foreground)' },
  { tag: tags.definition(tags.propertyName), color: 'var(--foreground)' },

  { tag: tags.atom, color: 'var(--info)' },
  { tag: tags.meta, color: 'var(--muted-foreground)' },
  { tag: tags.tagName, color: 'var(--primary)' },
  { tag: tags.attributeName, color: 'var(--accent)' },
  { tag: tags.attributeValue, color: 'var(--success)' },

  { tag: tags.heading, color: 'var(--primary)', fontWeight: '600' },
  { tag: tags.emphasis, fontStyle: 'italic' },
  { tag: tags.strong, fontWeight: '600' },
  { tag: tags.link, color: 'var(--info)', textDecoration: 'underline' },

  { tag: tags.invalid, color: 'var(--destructive)' },
])

export const shipHighlighting = syntaxHighlighting(shipHighlightStyle)
