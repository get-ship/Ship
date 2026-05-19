// Block-level parsers — {#if}/{#for}/{#match} and their headers.
// Shared by compile.ts via the small ParseScope interface. Kept in a
// sibling module to keep compile.ts under the file-size cap.

import { compileExpr, type CompiledExpr } from './expr'
import type { Instr } from './compile'
import type { Token } from './tokenize'

const WHILE_LOADING_TERMS = ['close-while-loading'] as const
const WHILE_ERROR_TERMS = ['close-while-error'] as const
const ASYNC_TERMS = ['async-loading', 'async-error', 'async-ok', 'close-async'] as const

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

/** Multi-instr result for blocks that desugar to >1 top-level instr. */
export interface InstrsResult {
  instrs: Instr[]
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

export function parseWhileLoading(
  tokens: Token[],
  start: number,
  scope: ParseScope,
): InstrResult {
  const head = tokens[start] as Extract<Token, { kind: 'open-while-loading' }>
  const m = head.src.match(/^of\s+([A-Za-z_$][\w$]*)$/)
  if (!m) {
    throw new Error(
      `[ship-set-dom] {#while-loading} expects "of <signal>" (at ${head.loc})`,
    )
  }
  const bind = m[1]!
  let i = start + 1
  const body = scope.body(tokens, i, WHILE_LOADING_TERMS)
  i = body.end
  const closer = tokens[i]
  if (!closer || closer.kind !== 'close-while-loading') {
    throw new Error(`[ship-set-dom] missing {/while-loading} (at ${head.loc})`)
  }
  return { instr: { kind: 'while-loading', bind, body: body.instrs }, end: i + 1 }
}

export function parseWhileError(
  tokens: Token[],
  start: number,
  scope: ParseScope,
): InstrResult {
  const head = tokens[start] as Extract<Token, { kind: 'open-while-error' }>
  const m = head.src.match(/^([A-Za-z_$][\w$]*)\s+of\s+([A-Za-z_$][\w$]*)$/)
  if (!m) {
    throw new Error(
      `[ship-set-dom] {#while-error} expects "<var> of <signal>" (at ${head.loc})`,
    )
  }
  const errorVar = m[1]!
  const bind = m[2]!
  let i = start + 1
  const body = scope.body(tokens, i, WHILE_ERROR_TERMS)
  i = body.end
  const closer = tokens[i]
  if (!closer || closer.kind !== 'close-while-error') {
    throw new Error(`[ship-set-dom] missing {/while-error} (at ${head.loc})`)
  }
  return { instr: { kind: 'while-error', bind, errorVar, body: body.instrs }, end: i + 1 }
}

/**
 * Parse `{#async <signal>[ as <var>]}` block. Desugars at parse-time into
 * the three existing template constructs:
 *
 *   {#while-loading of <signal>}   ← from {:loading} body, or default
 *     ...
 *   {/while-loading}
 *   {#while-error <err> of <signal>}   ← from {:error <err>} body, or default
 *     ...
 *   {/while-error}
 *   {#if <signal>().kind === 'ok'}     ← from {:ok} body (required)
 *     {let <var> = <signal>().value in ...}
 *   {/if}
 *
 * Branch bodies can appear in any order. `{:ok}` is mandatory; the other
 * two are optional and fall back to `<sui-spinner centered>` and
 * `<sui-error-state>{<err>.message}</sui-error-state>`. The `as <var>`
 * binding is reactive — every read re-evaluates the underlying signal.
 */
export function parseAsync(
  tokens: Token[],
  start: number,
  scope: ParseScope,
): InstrsResult {
  const head = tokens[start] as Extract<Token, { kind: 'open-async' }>
  const { signal, asVar } = parseAsyncHeader(head.src, head.loc)

  let i = start + 1
  let loadingBody: Instr[] | null = null
  let errorBody: Instr[] | null = null
  let errorVar = 'error'
  let okBody: Instr[] | null = null

  // Skip leading whitespace before the first branch marker.
  while (i < tokens.length) {
    const t = tokens[i]!
    if (t.kind === 'text' && t.value.trim() === '') {
      i++
      continue
    }
    break
  }

  while (i < tokens.length) {
    const t = tokens[i]!
    if (t.kind === 'close-async') {
      i++
      break
    }
    if (t.kind === 'async-loading') {
      i++
      const body = scope.body(tokens, i, ASYNC_TERMS)
      loadingBody = body.instrs
      i = body.end
      continue
    }
    if (t.kind === 'async-error') {
      const v = t.src.trim()
      if (v) {
        if (!/^[A-Za-z_$][\w$]*$/.test(v)) {
          throw new Error(
            `[ship-set-dom] {:error} bad variable: "${v}" (at ${t.loc})`,
          )
        }
        errorVar = v
      }
      i++
      const body = scope.body(tokens, i, ASYNC_TERMS)
      errorBody = body.instrs
      i = body.end
      continue
    }
    if (t.kind === 'async-ok') {
      i++
      const body = scope.body(tokens, i, ASYNC_TERMS)
      okBody = body.instrs
      i = body.end
      continue
    }
    throw new Error(
      `[ship-set-dom] unexpected token in {#async}: ${t.kind} (at ${head.loc})`,
    )
  }

  if (okBody === null) {
    throw new Error(
      `[ship-set-dom] {#async} requires {:ok} branch (at ${head.loc})`,
    )
  }

  // Build the three desugared instructions.
  const out: Instr[] = []
  const loc = head.loc

  out.push({
    kind: 'while-loading',
    bind: signal,
    body: loadingBody ?? defaultLoadingBody(),
  })

  out.push({
    kind: 'while-error',
    bind: signal,
    errorVar,
    body: errorBody ?? defaultErrorBody(errorVar, loc),
  })

  // Wrap the ok body in a `let` binding if `as v` was given, so that
  // `v.field` references inside the body resolve through the signal.
  const okWithLet: Instr[] = asVar
    ? [
        {
          kind: 'let',
          name: asVar,
          value: compileExpr(`${signal}().value`, { location: `${loc} as ${asVar}` }),
          body: okBody,
        },
      ]
    : okBody

  out.push({
    kind: 'if',
    branches: [
      {
        cond: compileExpr(`${signal}().kind === 'ok'`, { location: `${loc} :ok` }),
        body: okWithLet,
      },
    ],
  })

  return { instrs: out, end: i }
}

/** Parse `<signal>[ as <var>]` from the open-async token's src. */
function parseAsyncHeader(
  src: string,
  loc: string,
): { signal: string; asVar: string | null } {
  const trimmed = src.trim()
  const asIdx = findKeywordIndex(trimmed, ' as ')
  let signal = trimmed
  let asVar: string | null = null
  if (asIdx !== -1) {
    signal = trimmed.slice(0, asIdx).trim()
    asVar = trimmed.slice(asIdx + 4).trim()
    if (!/^[A-Za-z_$][\w$]*$/.test(asVar)) {
      throw new Error(
        `[ship-set-dom] {#async} bad "as" variable: "${asVar}" (at ${loc})`,
      )
    }
  }
  if (!/^[A-Za-z_$][\w$]*$/.test(signal)) {
    throw new Error(
      `[ship-set-dom] {#async} expects a bare signal name (got "${signal}") at ${loc}`,
    )
  }
  return { signal, asVar }
}

/** Default `{:loading}` body — `<sui-spinner size="md" centered>`. */
function defaultLoadingBody(): Instr[] {
  return [
    {
      kind: 'element',
      tag: 'sui-spinner',
      attrs: [
        { kind: 'static', name: 'size', value: 'md' },
        { kind: 'static', name: 'centered', value: '' },
      ],
      children: [],
    },
  ]
}

/** Default `{:error err}` body — `<sui-error-state>{err.message}</sui-error-state>`. */
function defaultErrorBody(errorVar: string, loc: string): Instr[] {
  return [
    {
      kind: 'element',
      tag: 'sui-error-state',
      attrs: [],
      children: [
        {
          kind: 'dynamic-text',
          expr: compileExpr(`${errorVar}.message`, {
            location: `${loc} :error default`,
          }),
        },
      ],
    },
  ]
}
