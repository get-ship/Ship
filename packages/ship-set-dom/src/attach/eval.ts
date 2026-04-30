// Scope evaluator.
//
// Compose a `(scope) => unknown` thunk from a raw expression by
// wrapping it in `with ($scope) return (expr)`. Mirrors the shape
// the compiled module's own `__scopeEval` uses (`crates/ship-set-
// compiler/src/emit/js.rs`) so that `expr.deps` semantics — "every
// identifier read at the top level of `with`" — match exactly.
//
// Evaluation is stateless: callers wrap the call in `effect()` from
// the runtime to register reactive deps. Reads of signal getters
// inside the expression auto-subscribe the surrounding effect.

const cache = new WeakMap<object, Map<string, ScopeEval>>()
const MODULE_KEY: object = {}

export type ScopeEval = (scope: Record<string, unknown>) => unknown

/**
 * Compile `expr` once per source string and reuse. Returns a function
 * that evaluates the expression with `scope` as the `with`-bound
 * lookup. Throws on syntactically invalid expressions; runtime errors
 * during evaluation surface to the caller.
 */
export function compileExpr(expr: string): ScopeEval {
  let perModule = cache.get(MODULE_KEY)
  if (!perModule) {
    perModule = new Map()
    cache.set(MODULE_KEY, perModule)
  }
  const hit = perModule.get(expr)
  if (hit) return hit
  // eslint-disable-next-line @typescript-eslint/no-implied-eval
  const fn = new Function(
    '$scope',
    'with ($scope) { return (' + expr + '); }',
  ) as ScopeEval
  perModule.set(expr, fn)
  return fn
}
