// Browser-safe structural diagnostics.
//
// Wraps `@ship/sui-lsp/parse` (a pure parser with zero Node deps)
// and converts its `ParseError[]` into the `SuiDiagnostic[]` shape
// `<sui-code-editor>.setDiagnostics()` expects.
//
// "Structural" here means: parse errors, block-balance errors,
// malformed attributes, unterminated braces. Things the parser
// itself catches. Type-aware diagnostics (identifier resolution,
// signal scope) live in the Node LSP and are not yet ported.

import { parse } from '@ship/sui-lsp/parse'
import type { ParseError } from '@ship/sui-lsp/parse-types'

/** Shape consumed by `<sui-code-editor>.setDiagnostics()`. */
export interface SuiDiagnostic {
  line: number
  column?: number
  endLine?: number
  endColumn?: number
  severity: 'error' | 'warning' | 'info'
  message: string
  source?: string
}

/** Convert a byte offset to a 1-based line/col pair against the
 *  original source. Tracks newline counts; no allocation per call. */
function offsetToLineCol(source: string, offset: number): { line: number; column: number } {
  const safe = Math.max(0, Math.min(offset, source.length))
  let line = 1
  let lastNewline = -1
  for (let i = 0; i < safe; i++) {
    if (source.charCodeAt(i) === 10 /* \n */) {
      line++
      lastNewline = i
    }
  }
  return { line, column: safe - lastNewline }
}

/** Parse a `.sui` document and emit one diagnostic per ParseError. */
export function diagnose(source: string): SuiDiagnostic[] {
  const { errors } = parse(source)
  return errors.map((e: ParseError) => {
    const start = offsetToLineCol(source, e.range.start)
    const end = offsetToLineCol(source, e.range.end)
    return {
      line: start.line,
      column: start.column,
      endLine: end.line,
      endColumn: end.column,
      severity: 'error',
      message: e.message,
      source: 'sui-parse',
    }
  })
}
