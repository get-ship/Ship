// Block-level parsers — {#if}/{#for}/{#match} and their headers.
// Shared by compile.ts via the small ParseScope interface. Kept in a
// sibling module to keep compile.ts under the file-size cap.

import { compileExpr, type CompiledExpr } from './expr'
import type { Instr } from './compile'
import type { Token } from './tokenize'

export interface ParseScope {
  /** Parse children up to one of the listed terminators. */
  body: (
    tokens: Token[],
    start: number,
    terminators: ReadonlyArray<Token['kind']>,
  ) => { instrs: Instr[]; end: number }
}

export interface InstrResult {
  instr: Instr
  end: number
}

const IF_TERMS = ['else-if', 'else', 'close-if'] as const
const FOR_TERMS = ['else', 'close-for'] as const
const MATCH_TERMS = ['case', 'default', 'close-match'] as const

export function parseIf(
  tokens: Token[],
  start: number,
  scope: ParseScope,
): InstrResult {
  const head = tokens[start] as Extract<Token, { kind: 'open-if' }>
  const branches: { cond: CompiledExpr; body: Instr[] }[] = []
  let else_: Instr[] | undefined
  let cur = compileExpr(head.src, { location: head.loc })
  let i = start + 1
  while (true) {
    const body = scope.body(tokens, i, IF_TERMS)
    branches.push({ cond: cur, body: body.instrs })
    i = body.end
    const term = tokens[i]
    if (!term) {
      throw new Error(`[ship-set-dom] missing {/if} (at ${head.loc})`)
    }
    if (term.kind === 'close-if') {
      i++
      break
    }
    if (term.kind === 'else-if') {
      cur = compileExpr(term.src, { location: term.loc })
      i++
      continue
    }
    if (term.kind === 'else') {
      i++
      const elseBody = scope.body(tokens, i, ['close-if'])
      else_ = elseBody.instrs
      i = elseBody.end
      const closer = tokens[i]
      if (!closer || closer.kind !== 'close-if') {
        throw new Error(`[ship-set-dom] missing {/if} (at ${head.loc})`)
      }
      i++
      break
    }
  }
  const instr: Instr =
    else_ !== undefined
      ? { kind: 'if', branches, else_ }
      : { kind: 'if', branches }
  return { instr, end: i }
}

export function parseFor(
  tokens: Token[],
  start: number,
  scope: ParseScope,
): InstrResult {
  const head = tokens[start] as Extract<Token, { kind: 'open-for' }>
  const { varName, indexName, iterable, key } = parseForHeader(
    head.src,
    head.loc,
  )
  let i = start + 1
  const body = scope.body(tokens, i, FOR_TERMS)
  i = body.end
  let else_: Instr[] | undefined
  const term = tokens[i]
  if (term?.kind === 'else') {
    i++
    const elseBody = scope.body(tokens, i, ['close-for'])
    else_ = elseBody.instrs
    i = elseBody.end
  }
  const closer = tokens[i]
  if (!closer || closer.kind !== 'close-for') {
    throw new Error(`[ship-set-dom] missing {/for} (at ${head.loc})`)
  }
  i++
  const instr: Instr = {
    kind: 'for',
    varName,
    indexName,
    iterable,
    key,
    body: body.instrs,
    ...(else_ !== undefined ? { else_ } : {}),
  }
  return { instr, end: i }
}

function parseForHeader(
  src: string,
  loc: string,
): {
  varName: string
  indexName: string | null
  iterable: CompiledExpr
  key: CompiledExpr | null
} {
  // shape: `<binding> of <iterable>[ key={<expr>}]`
  const ofIdx = findKeywordIndex(src, ' of ')
  if (ofIdx === -1) {
    throw new Error(`[ship-set-dom] {#for} expects "X of items" (at ${loc})`)
  }
  const left = src.slice(0, ofIdx).trim()
  let right = src.slice(ofIdx + 4).trim()

  let varName = left
  let indexName: string | null = null
  // Support `(x, i)` form for index access.
  const parenMatch = left.match(
    /^\(\s*([A-Za-z_$][\w$]*)\s*,\s*([A-Za-z_$][\w$]*)\s*\)$/,
  )
  if (parenMatch) {
    varName = parenMatch[1]!
    indexName = parenMatch[2]!
  }
  if (!/^[A-Za-z_$][\w$]*$/.test(varName)) {
    throw new Error(`[ship-set-dom] {#for} bad variable: ${left} (at ${loc})`)
  }

  let key: CompiledExpr | null = null
  const keyMatch = right.match(/\skey=\{(.+)\}\s*$/s)
  if (keyMatch) {
    key = compileExpr(keyMatch[1]!, { location: `${loc} key=` })
    right = right.slice(0, keyMatch.index).trim()
  }
  const iterable = compileExpr(right, { location: `${loc} of ${right}` })
  return { varName, indexName, iterable, key }
}

/** Find " of " (or any keyword) ignoring matches inside strings/parens. */
function findKeywordIndex(s: string, keyword: string): number {
  let str: '"' | "'" | '`' | null = null
  let depth = 0
  for (let i = 0; i <= s.length - keyword.length; i++) {
    const c = s[i]!
    if (str) {
      if (c === '\\') {
        i++
        continue
      }
      if (c === str) str = null
      continue
    }
    if (c === '"' || c === "'" || c === '`') {
      str = c
      continue
    }
    if (c === '(' || c === '[' || c === '{') depth++
    else if (c === ')' || c === ']' || c === '}') depth--
    if (depth === 0 && s.slice(i, i + keyword.length) === keyword) return i
  }
  return -1
}

export function parseMatch(
  tokens: Token[],
  start: number,
  scope: ParseScope,
): InstrResult {
  const head = tokens[start] as Extract<Token, { kind: 'open-match' }>
  const discriminant = compileExpr(head.src, { location: head.loc })
  let i = start + 1
  // Skip leading whitespace tokens before the first {:case}.
  while (i < tokens.length) {
    const t = tokens[i]!
    if (t.kind === 'text' && t.value.trim() === '') {
      i++
      continue
    }
    break
  }
  const cases: { test: CompiledExpr; body: Instr[] }[] = []
  let default_: Instr[] | undefined
  while (i < tokens.length) {
    const t = tokens[i]!
    if (t.kind === 'close-match') {
      i++
      break
    }
    if (t.kind === 'case') {
      const test = compileExpr(t.src, { location: t.loc })
      i++
      const body = scope.body(tokens, i, MATCH_TERMS)
      cases.push({ test, body: body.instrs })
      i = body.end
      continue
    }
    if (t.kind === 'default') {
      i++
      const body = scope.body(tokens, i, ['close-match'])
      default_ = body.instrs
      i = body.end
      continue
    }
    throw new Error(
      `[ship-set-dom] unexpected token in {#match}: ${t.kind} (at ${head.loc})`,
    )
  }
  const instr: Instr = {
    kind: 'match',
    discriminant,
    cases,
    ...(default_ !== undefined ? { default_ } : {}),
  }
  return { instr, end: i }
}
