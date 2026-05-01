// Controller-section analyzers extending setup.ts.
//
// Two D-arc capabilities live here:
//   • `use:` namespace extraction — for each `use:` alias, walk the
//     hook factory's return type to enumerate the contributions
//     exposed under the alias (state / derived / on / use). Drives
//     hover, completion, and member-validation diagnostics.
//   • `on:` handler extraction — capture `on:` section keys with
//     a textual signature (parameter list + return) for handler
//     completion and signature display in `on:click={…}` braces.
//
// Both walk the same `controller(def)` argument object literal already
// resolved by setup.ts.

import ts from 'typescript'
import type {
  OnHandler,
  UseNamespace,
  UseNamespaceMember,
} from './setup'

/** Walk the `use:` section, building one `UseNamespace` per alias. */
export function namespacesFromControllerDef(
  obj: ts.ObjectLiteralExpression,
  sf: ts.SourceFile,
): UseNamespace[] {
  const useProp = obj.properties.find(
    (p) => ts.isPropertyAssignment(p) && ts.isIdentifier(p.name) && p.name.text === 'use',
  )
  if (!useProp || !ts.isPropertyAssignment(useProp)) return []
  const useObj = unwrapToObject(useProp.initializer)
  if (!useObj) return []
  const out: UseNamespace[] = []
  for (const member of useObj.properties) {
    if (!ts.isPropertyAssignment(member) || !ts.isIdentifier(member.name)) continue
    const alias = member.name.text
    const ns = resolveNamespace(alias, member.initializer, sf)
    if (ns) out.push(ns)
  }
  return out
}

/** Walk the `on:` section, capturing each handler's textual signature. */
export function onHandlersFromControllerDef(
  obj: ts.ObjectLiteralExpression,
  sf: ts.SourceFile,
): OnHandler[] {
  const onProp = obj.properties.find(
    (p) => ts.isPropertyAssignment(p) && ts.isIdentifier(p.name) && p.name.text === 'on',
  )
  if (!onProp || !ts.isPropertyAssignment(onProp)) return []
  const onObj = unwrapToObject(onProp.initializer)
  if (!onObj) return []
  const out: OnHandler[] = []
  for (const m of onObj.properties) {
    const named = namedMember(m)
    if (!named) continue
    out.push({ name: named.name, signature: handlerSignature(m, sf) })
  }
  return out
}

function resolveNamespace(
  alias: string,
  initializer: ts.Expression,
  sf: ts.SourceFile,
): UseNamespace | null {
  // Inline literal: `use: { list: { state: {...}, derived: {...} } }`.
  const inline = unwrapToObject(initializer)
  if (inline) {
    return { alias, hookFactory: null, members: hookContributions(inline, sf) }
  }
  // Hook factory call: `use: { list: useFilteredList(items, pred) }`.
  if (!ts.isCallExpression(initializer)) return null
  const callee = initializer.expression
  if (!ts.isIdentifier(callee)) return null
  const factoryName = callee.text
  const factoryDecl = findDeclarationInSource(sf, factoryName)
  if (!factoryDecl) {
    return { alias, hookFactory: factoryName, members: [] }
  }
  const fnObj = returnedObjectLiteral(factoryDecl)
  if (!fnObj) {
    return { alias, hookFactory: factoryName, members: [] }
  }
  return { alias, hookFactory: factoryName, members: hookContributions(fnObj, sf) }
}

/** Pull contributions out of a hook's returned object literal. */
function hookContributions(
  obj: ts.ObjectLiteralExpression,
  sf: ts.SourceFile,
): UseNamespaceMember[] {
  const out: UseNamespaceMember[] = []
  for (const sectionName of ['state', 'derived', 'on', 'use'] as const) {
    const prop = obj.properties.find(
      (p) =>
        ts.isPropertyAssignment(p) && ts.isIdentifier(p.name) && p.name.text === sectionName,
    )
    if (!prop || !ts.isPropertyAssignment(prop)) continue
    const inner = unwrapToObject(prop.initializer)
    if (!inner) continue
    for (const m of inner.properties) {
      const named = namedMember(m)
      if (!named) continue
      out.push({
        name: named.name,
        kind: sectionName,
        signature: contributionSignature(sectionName, m, sf),
      })
    }
  }
  return out
}

function namedMember(m: ts.ObjectLiteralElementLike): { name: string } | null {
  if (
    (ts.isPropertyAssignment(m) ||
      ts.isShorthandPropertyAssignment(m) ||
      ts.isMethodDeclaration(m)) &&
    m.name &&
    ts.isIdentifier(m.name)
  ) {
    return { name: m.name.text }
  }
  return null
}

function contributionSignature(
  section: 'state' | 'derived' | 'on' | 'use',
  m: ts.ObjectLiteralElementLike,
  sf: ts.SourceFile,
): string {
  if (section === 'derived') return `() => ${derivedReturnText(m, sf)}`
  if (section === 'on') return handlerSignature(m, sf)
  if (section === 'state') return stateValueText(m, sf)
  return 'namespace'
}

function derivedReturnText(m: ts.ObjectLiteralElementLike, sf: ts.SourceFile): string {
  const fn = functionFromMember(m)
  if (!fn) return 'unknown'
  if (fn.type) return fn.type.getText(sf)
  return 'unknown'
}

function stateValueText(m: ts.ObjectLiteralElementLike, sf: ts.SourceFile): string {
  if (ts.isPropertyAssignment(m)) return `Signal<${typeofExpr(m.initializer, sf)}>`
  return 'Signal<unknown>'
}

function typeofExpr(e: ts.Expression, sf: ts.SourceFile): string {
  if (ts.isStringLiteral(e) || ts.isNoSubstitutionTemplateLiteral(e)) return 'string'
  if (e.kind === ts.SyntaxKind.TrueKeyword || e.kind === ts.SyntaxKind.FalseKeyword) return 'boolean'
  if (ts.isNumericLiteral(e)) return 'number'
  if (ts.isArrayLiteralExpression(e)) return 'unknown[]'
  if (ts.isObjectLiteralExpression(e)) return 'object'
  return e.getText(sf)
}

/** Render `(arg1: T, arg2: U) => R` from a method/property handler. */
export function handlerSignature(
  m: ts.ObjectLiteralElementLike,
  sf: ts.SourceFile,
): string {
  const fn = functionFromMember(m)
  if (!fn) return '(...args) => void'
  const params = fn.parameters
    .map((p) => p.getText(sf))
    .join(', ')
  const ret = fn.type ? fn.type.getText(sf) : 'void'
  return `(${params}) => ${ret}`
}

function functionFromMember(
  m: ts.ObjectLiteralElementLike,
):
  | ts.ArrowFunction
  | ts.FunctionExpression
  | ts.MethodDeclaration
  | null {
  if (ts.isMethodDeclaration(m)) return m
  if (ts.isPropertyAssignment(m)) {
    const init = m.initializer
    if (ts.isArrowFunction(init) || ts.isFunctionExpression(init)) return init
  }
  return null
}

function returnedObjectLiteral(decl: ts.Node): ts.ObjectLiteralExpression | null {
  // The factory's *body* — find the final return-statement object.
  if (
    ts.isFunctionDeclaration(decl) ||
    ts.isArrowFunction(decl) ||
    ts.isFunctionExpression(decl)
  ) {
    if (ts.isArrowFunction(decl) && !ts.isBlock(decl.body)) {
      return unwrapToObject(decl.body)
    }
    const body = decl.body as ts.Block | undefined
    if (!body) return null
    for (let i = body.statements.length - 1; i >= 0; i--) {
      const s = body.statements[i]!
      if (ts.isReturnStatement(s) && s.expression) return unwrapToObject(s.expression)
    }
    return null
  }
  return null
}

function findDeclarationInSource(
  sf: ts.SourceFile,
  name: string,
): ts.Node | null {
  for (const stmt of sf.statements) {
    if (ts.isFunctionDeclaration(stmt) && stmt.name?.text === name) return stmt
    if (ts.isVariableStatement(stmt)) {
      for (const decl of stmt.declarationList.declarations) {
        if (ts.isIdentifier(decl.name) && decl.name.text === name && decl.initializer) {
          return decl.initializer
        }
      }
    }
  }
  return null
}

function unwrapToObject(expr: ts.Node): ts.ObjectLiteralExpression | null {
  if (ts.isObjectLiteralExpression(expr)) return expr
  if (ts.isParenthesizedExpression(expr)) return unwrapToObject(expr.expression)
  if (ts.isArrowFunction(expr) || ts.isFunctionExpression(expr)) {
    if (!ts.isBlock(expr.body)) return unwrapToObject(expr.body)
    const body = expr.body
    for (let i = body.statements.length - 1; i >= 0; i--) {
      const s = body.statements[i]!
      if (ts.isReturnStatement(s) && s.expression) return unwrapToObject(s.expression)
    }
  }
  return null
}
