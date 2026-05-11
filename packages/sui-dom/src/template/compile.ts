// Compile token streams (and recursive element subtrees) into a tree
// of instructions. The result is a pure data structure — render.ts
// turns it into actual DOM at hydrate time.

import { compileExpr, type CompiledExpr } from './expr'
import { compileAttrs, type AttrInstr } from '../bindings/attr-compile'
import { tokenizeChildren, type Token } from './tokenize'
import { parseIf, parseFor, parseMatch, parseWhileLoading, parseWhileError, type ParseScope } from './compile-blocks'

/** One unit of rendered output. */
export type Instr =
  | { kind: 'static-text'; text: string }
  | { kind: 'dynamic-text'; expr: CompiledExpr }
  | { kind: 'comment'; data: string }
  | { kind: 'element'; tag: string; attrs: AttrInstr[]; children: Instr[] }
  | {
      kind: 'if'
      branches: { cond: CompiledExpr; body: Instr[] }[]
      else_?: Instr[]
    }
  | {
      kind: 'for'
      varName: string
      indexName: string | null
      iterable: CompiledExpr
      key: CompiledExpr | null
      body: Instr[]
      else_?: Instr[]
    }
  | {
      kind: 'match'
      discriminant: CompiledExpr
      cases: { test: CompiledExpr; body: Instr[] }[]
      default_?: Instr[]
    }
  | { kind: 'while-loading'; bind: string; body: Instr[] }
  | { kind: 'while-error'; bind: string; errorVar: string; body: Instr[] }

/** Compile the children of `parent` into an instruction list. */
export function compileChildren(parent: Node): Instr[] {
  const tokens = tokenizeChildren(parent)
  return parseTokens(tokens, 0).instrs
}

interface ParseResult {
  instrs: Instr[]
  end: number
}

const blockScope: ParseScope = {
  body: (tokens, start, terms) => parseTokens(tokens, start, terms),
}

/**
 * Walk a flat token list and build the tree. `terminators` lists the
 * token kinds that end the current scope — encountering one returns
 * the parsed body up to it without consuming it.
 */
function parseTokens(
  tokens: Token[],
  start: number,
  terminators: ReadonlyArray<Token['kind']> = [],
): ParseResult {
  const instrs: Instr[] = []
  let i = start
  while (i < tokens.length) {
    const tok = tokens[i]!
    if (terminators.includes(tok.kind)) return { instrs, end: i }

    if (tok.kind === 'text') {
      if (tok.value.length > 0) instrs.push({ kind: 'static-text', text: tok.value })
      i++
      continue
    }
    if (tok.kind === 'comment') {
      instrs.push({ kind: 'comment', data: tok.node.data })
      i++
      continue
    }
    if (tok.kind === 'expr') {
      instrs.push({
        kind: 'dynamic-text',
        expr: compileExpr(tok.src, { location: tok.loc }),
      })
      i++
      continue
    }
    if (tok.kind === 'element') {
      instrs.push(compileElement(tok.node))
      i++
      continue
    }
    if (tok.kind === 'open-if') {
      const { instr, end } = parseIf(tokens, i, blockScope)
      instrs.push(instr)
      i = end
      continue
    }
    if (tok.kind === 'open-for') {
      const { instr, end } = parseFor(tokens, i, blockScope)
      instrs.push(instr)
      i = end
      continue
    }
    if (tok.kind === 'open-match') {
      const { instr, end } = parseMatch(tokens, i, blockScope)
      instrs.push(instr)
      i = end
      continue
    }
    if (tok.kind === 'open-while-loading') {
      const { instr, end } = parseWhileLoading(tokens, i, blockScope)
      instrs.push(instr)
      i = end
      continue
    }
    if (tok.kind === 'open-while-error') {
      const { instr, end } = parseWhileError(tokens, i, blockScope)
      instrs.push(instr)
      i = end
      continue
    }
    throw new Error(
      `[ship-set-dom] unexpected token: ${tok.kind} (at ${getLoc(tok)})`,
    )
  }
  return { instrs, end: i }
}

const getLoc = (tok: Token): string =>
  'loc' in tok && typeof tok.loc === 'string' ? tok.loc : '?'

function compileElement(el: Element): Instr {
  const tag = el.tagName.toLowerCase()
  return {
    kind: 'element',
    tag: el.tagName, // preserve original casing for the renderer
    attrs: compileAttrs(el, `<${tag}>`),
    children: compileChildren(el),
  }
}
