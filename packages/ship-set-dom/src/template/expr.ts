// Expression compiler for {expr} interpolation, attr={x}, on:event={fn},
// and the value/source attributes of <state>/<data>.
//
// Inputs are JS-shaped expression strings; outputs are functions of a
// scope object. We compile via `new Function`, which produces a
// non-strict body — that lets us use `with($scope) { return (expr) }`
// to look up identifiers without enumerating scope keys at compile time.
//
// Validation rejects statements, assignments, and (optionally) arrows.
// The interpreter's safety story is "expressions can read scope, call
// scope-visible functions, and that's it" — anything else throws at
// compile time with a helpful pointer to the offending span.

/** A compiled expression — call with the scope object to evaluate. */
export type CompiledExpr = (scope: Record<string, unknown>) => unknown

export interface CompileExprOptions {
  /** Allow arrow functions (used for `on:event={() => ...}` bindings). */
  allowArrow?: boolean
  /** Source-location hint for error messages. */
  location?: string
}

/**
 * Compile a `{expr}`-style template expression into a function.
 *
 * The resulting function takes a scope object and returns the
 * expression's value. Identifier reads resolve through `with(scope)`,
 * so any name visible on the scope object can be used; missing names
 * throw `ReferenceError` at evaluation time (consistent with JS).
 *
 * Rejects at compile time:
 *  - statements (`;` outside of string/template literals)
 *  - assignments (`=` not part of `==`/`!=`/`>=`/`<=`/`=>`/`===`/`!==`)
 *  - arrow functions, unless `opts.allowArrow` is `true`
 *
 * @param expr Raw expression source — no surrounding `{` `}`.
 * @param opts Compile options (allow arrows, error-location hint).
 *
 * @example
 * ```ts
 * const fn = compileExpr('count()')
 * fn({ count: () => 7 })   // → 7
 * ```
 */
export function compileExpr(
  expr: string,
  opts: CompileExprOptions = {},
): CompiledExpr {
  const trimmed = expr.trim()
  if (!trimmed) {
    throw new SyntaxError(formatErr('empty expression', opts.location))
  }
  validate(trimmed, opts)
  const body = `with($scope) { return (${trimmed}); }`
  let fn: (s: Record<string, unknown>) => unknown
  try {
    fn = new Function('$scope', body) as typeof fn
  } catch (e) {
    throw new SyntaxError(
      formatErr(
        `invalid expression — ${(e as Error).message}: \`${trimmed}\``,
        opts.location,
      ),
    )
  }
  return (scope) => fn(scope)
}

/**
 * Compile a source attribute (e.g. `<data source="ship.x.y()">`) into a
 * thunk over named runtime parameters. The body runs with `with` scope
 * so any locally-known names also resolve.
 */
export function compileSource(
  expr: string,
  paramNames: readonly string[],
  opts: CompileExprOptions = {},
): (...args: unknown[]) => unknown {
  validate(expr.trim(), opts)
  const body = `return (${expr.trim()});`
  try {
    return new Function(...paramNames, body) as (...a: unknown[]) => unknown
  } catch (e) {
    throw new SyntaxError(
      formatErr(
        `invalid source — ${(e as Error).message}: \`${expr.trim()}\``,
        opts.location,
      ),
    )
  }
}

const formatErr = (msg: string, loc?: string): string =>
  loc ? `[ship-set-dom] ${msg} (at ${loc})` : `[ship-set-dom] ${msg}`

interface ScanState {
  inString: '"' | "'" | '`' | null
  inTemplate: number
  parenDepth: number
  braceDepth: number
}

/**
 * Validate an expression source for forbidden syntax. Walks the string
 * char-by-char tracking string/template-literal state so the checks
 * don't fire on contents inside `'='` or `` `=>` `` literals.
 */
function validate(expr: string, opts: CompileExprOptions): void {
  const len = expr.length
  const state: ScanState = {
    inString: null,
    inTemplate: 0,
    parenDepth: 0,
    braceDepth: 0,
  }

  for (let i = 0; i < len; i++) {
    const c = expr[i]!
    const prev = i > 0 ? expr[i - 1] : ''
    const next = i + 1 < len ? expr[i + 1] : ''

    if (state.inString) {
      if (c === '\\') {
        i++
        continue
      }
      if (c === state.inString) {
        state.inString = null
      } else if (state.inString === '`' && c === '$' && next === '{') {
        state.inTemplate++
        state.inString = null
        i++
      }
      continue
    }
    if (c === '"' || c === "'" || c === '`') {
      state.inString = c
      continue
    }
    if (c === '{') {
      state.braceDepth++
      continue
    }
    if (c === '}') {
      if (state.inTemplate > 0 && state.braceDepth === 0) {
        state.inTemplate--
        state.inString = '`'
        continue
      }
      state.braceDepth--
      continue
    }
    if (c === '(') state.parenDepth++
    if (c === ')') state.parenDepth--

    if (c === ';') {
      throw new SyntaxError(
        formatErr(
          `statements are not allowed in expressions: \`${expr}\``,
          opts.location,
        ),
      )
    }
    if (c === '=' && next === '>') {
      if (!opts.allowArrow) {
        throw new SyntaxError(
          formatErr(
            `arrow functions are not allowed here: \`${expr}\``,
            opts.location,
          ),
        )
      }
      i++
      continue
    }
    if (c === '=') {
      const prev1 = prev
      const next1 = next
      const isCompare =
        prev1 === '=' ||
        prev1 === '!' ||
        prev1 === '<' ||
        prev1 === '>' ||
        next1 === '='
      if (!isCompare) {
        throw new SyntaxError(
          formatErr(
            `assignments are not allowed in expressions: \`${expr}\``,
            opts.location,
          ),
        )
      }
    }
  }

  if (state.inString) {
    throw new SyntaxError(
      formatErr(`unterminated string in expression: \`${expr}\``, opts.location),
    )
  }
}
