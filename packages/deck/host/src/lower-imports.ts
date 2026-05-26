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
