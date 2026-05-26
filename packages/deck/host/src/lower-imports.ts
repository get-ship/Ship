//! Confined-build transform: lower framework ESM imports to free globals.
//!
//! The SES runner evaluates a controller as source in a Compartment whose
//! framework surface (`signal`, `controller`, `effect`, …) is ENDOWED as
//! globals. The Compartment module loader is proven friction (spike #2), so the
//! confined build instead strips the controller's
//! `import { signal } from '@ship/sui-runtime'` lines — the named bindings then
//! resolve as the free globals the runner endows.
//!
//! Operates on BUILT controller JS (types already erased by tsc/bundler). Only
//! the SUI framework specifiers are lowered; every other import (a pack, a
//! relative module) is left intact — those are the controller's own deps, not
//! the endowed surface.

/// Strip `import … from '@ship/{sui-runtime|sui-dom|sdk-client}[/subpath]'`
/// (named, default, namespace, or side-effect) so the imported bindings resolve
/// as Compartment globals. Non-framework imports are untouched.
export function lowerFrameworkImports(source: string): string {
  // Matches an import statement whose specifier is a framework module:
  //   import { a, b as c } from '@ship/sui-runtime'
  //   import * as rt from '@ship/sui-dom'
  //   import '@ship/sui-runtime/template-hooks'   (side-effect)
  // The optional `(?:…\s+from\s+)?` clause covers the binding part (which never
  // contains a quote), so side-effect imports (no `from`) also match.
  const FRAMEWORK_IMPORT =
    /^[ \t]*import\s+(?:[^'"]*?\s+from\s+)?['"]@ship\/(?:sui-runtime(?:\/[\w-]+)?|sui-dom|sdk-client)['"][ \t]*;?[ \t]*\r?\n?/gm
  return source.replace(FRAMEWORK_IMPORT, '')
}

/**
 * Lower a built view MODULE to a self-contained expression the SES runner can
 * `Compartment.evaluate()` — which runs source as a script, so ESM `export`
 * syntax must go. We confine the WHOLE compiled view (glue + bundled
 * controller): there's no trust boundary between them, and the glue is just the
 * DOM renderer + runtime wiring, all reachable through endowments (a narrowed
 * node-factory `document`, the runtime surface, the policy-gated ops).
 *
 * Transforms (after {@link lowerFrameworkImports}):
 *   - `export default EXPR`        → `const __sui_default__ = EXPR` (+ a trailing
 *     `__sui_default__;` so evaluate returns it),
 *   - `export const|let|var|function|class …` → strip the `export ` keyword
 *     (the binding stays a module-local),
 *   - `export { … } [from '…']`    → removed.
 *
 * Returns source whose final expression is the module's default export, or the
 * source unchanged (sans framework imports) when there is no default.
 */
export function lowerModuleToExpression(source: string): string {
  let s = lowerFrameworkImports(source)
  let hasDefault = false
  s = s.replace(/^([ \t]*)export\s+default\s+/m, (_m, indent: string) => {
    hasDefault = true
    return `${indent}const __sui_default__ = `
  })
  s = s.replace(/^([ \t]*)export\s+(?=(?:const|let|var|function|class|async)\b)/gm, '$1')
  s = s.replace(/^[ \t]*export\s*\{[^}]*\}\s*(?:from\s*['"][^'"]*['"])?[ \t]*;?[ \t]*\r?\n?/gm, '')
  if (hasDefault) s += '\n;__sui_default__;\n'
  return s
}
