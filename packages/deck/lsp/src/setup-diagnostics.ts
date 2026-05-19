// Setup-file diagnostics emitted alongside `.sui` analysis.
//
// Two producers:
//   • derived-write (D-3) — `derived` sections receive a readonly view of
//     state; assignments are bugs. tsc catches them via the
//     `ReadonlyState<…>` parameter, but only on full type-check. This
//     module surfaces the same finding immediately as a Warning.
//   • empty-controller — a `controller({})` factory call (or `export
//     default null`) whose argument carries no behavior. Controllers
//     are optional in SUI; an empty file adds ceremony with no payoff,
//     so we flag it and recommend deleting the file.
//
// Output is the LSP's standard `Diagnostic` shape so the server can
// forward it under the setup file's URI without further translation.

import ts from 'typescript'
import * as path from 'node:path'
import type { Diagnostic } from './diagnostics'

const DERIVED_WRITE_MESSAGE =
  "derived sections cannot mutate state — move the assignment to an 'on' handler or 'mount'"

const EMPTY_CONTROLLER_MESSAGE =
  'Empty controller — delete this file. Components without behavior do not need a controller.'

export function diagnoseSetup(setupPath: string, source: string): Diagnostic[] {
  const sf = ts.createSourceFile(
    path.basename(setupPath),
    source,
    ts.ScriptTarget.ESNext,
    true,
    ts.ScriptKind.TS,
  )
  const out: Diagnostic[] = []
  const emptyDiag = diagnoseEmptyController(sf)
  if (emptyDiag) out.push(emptyDiag)
  const def = findDefaultControllerCall(sf)
  if (!def) return out
  const arg = def.arguments[0]
  if (!arg) return out
  const controllerDef = unwrapToObject(arg)
  if (!controllerDef) return out
  const derivedProp = controllerDef.properties.find(
    (p) => ts.isPropertyAssignment(p) && ts.isIdentifier(p.name) && p.name.text === 'derived',
  )
  if (!derivedProp || !ts.isPropertyAssignment(derivedProp)) return out
  const derivedObj = unwrapToObject(derivedProp.initializer)
  if (!derivedObj) return out
  for (const member of derivedObj.properties) {
    const fn = functionInitializer(member)
    if (!fn) continue
    walkForAssignments(fn, sf, out)
  }
  return out
}

// Flag `export default null` and `export default controller({...})` calls
// whose argument carries no behavior. The shapes recognized as "empty":
//   • `null` / `undefined` default export
//   • `controller()` — no argument
//   • `controller({})` — empty object literal
//   • `controller({ scope: () => ({}) })` — scope returns an empty object
//   • Every present member is itself empty: `scope` returns `{}`,
//     `derived` / `on` / `use` are `{}`, `mount` body is empty.
function diagnoseEmptyController(sf: ts.SourceFile): Diagnostic | null {
  const exportAssign = findDefaultExport(sf)
  if (!exportAssign) return null
  const expr = exportAssign.expression
  // `export default null` / `export default undefined`
  if (
    expr.kind === ts.SyntaxKind.NullKeyword ||
    (ts.isIdentifier(expr) && expr.text === 'undefined')
  ) {
    return {
      range: { start: exportAssign.getStart(sf), end: exportAssign.getEnd() },
      message: EMPTY_CONTROLLER_MESSAGE,
      severity: 'warning',
      source: 'sui-lsp',
      code: 'controller.empty',
    }
  }
  if (!ts.isCallExpression(expr)) return null
  if (!ts.isIdentifier(expr.expression) || expr.expression.text !== 'controller') return null
  const arg = expr.arguments[0]
  let isEmpty = false
  if (!arg) {
    isEmpty = true
  } else {
    const obj = unwrapToObjectLiteral(arg)
    if (!obj) return null
    if (obj.properties.length === 0) isEmpty = true
    else isEmpty = obj.properties.every(isEmptyControllerMember)
  }
  if (!isEmpty) return null
  return {
    range: { start: expr.getStart(sf), end: expr.getEnd() },
    message: EMPTY_CONTROLLER_MESSAGE,
    severity: 'warning',
    source: 'sui-lsp',
    code: 'controller.empty',
  }
}

function isEmptyControllerMember(m: ts.ObjectLiteralElementLike): boolean {
  const name = m.name && ts.isIdentifier(m.name) ? m.name.text : null
  if (!name) return false
  if (name === 'scope') {
    const fn = functionInitializer(m)
    if (!fn) {
      if (ts.isPropertyAssignment(m)) {
        const obj = unwrapToObjectLiteral(m.initializer)
        return !!obj && obj.properties.length === 0
      }
      return false
    }
    return isEmptyFunctionReturningObject(fn)
  }
  if (name === 'derived' || name === 'on' || name === 'use') {
    if (!ts.isPropertyAssignment(m)) return false
    const obj = unwrapToObjectLiteral(m.initializer)
    return !!obj && obj.properties.length === 0
  }
  if (name === 'mount') {
    const fn = functionInitializer(m)
    if (!fn) return false
    if (!fn.body) return false
    if (ts.isBlock(fn.body)) return fn.body.statements.length === 0
    return false
  }
  return false
}

function isEmptyFunctionReturningObject(
  fn: ts.ArrowFunction | ts.FunctionExpression | ts.MethodDeclaration,
): boolean {
  if (!fn.body) return false
  if (!ts.isBlock(fn.body)) {
    const obj = unwrapToObjectLiteral(fn.body)
    return !!obj && obj.properties.length === 0
  }
  if (fn.body.statements.length === 0) return true
  if (fn.body.statements.length !== 1) return false
  const stmt = fn.body.statements[0]!
  if (!ts.isReturnStatement(stmt) || !stmt.expression) return false
  const obj = unwrapToObjectLiteral(stmt.expression)
  return !!obj && obj.properties.length === 0
}

function findDefaultExport(sf: ts.SourceFile): ts.ExportAssignment | null {
  for (const stmt of sf.statements) {
    if (ts.isExportAssignment(stmt) && !stmt.isExportEquals) return stmt
  }
  return null
}

function unwrapToObjectLiteral(expr: ts.Node): ts.ObjectLiteralExpression | null {
  if (ts.isParenthesizedExpression(expr)) return unwrapToObjectLiteral(expr.expression)
  if (ts.isObjectLiteralExpression(expr)) return expr
  return null
}

function functionInitializer(
  m: ts.ObjectLiteralElementLike,
): ts.ArrowFunction | ts.FunctionExpression | ts.MethodDeclaration | null {
  if (ts.isMethodDeclaration(m)) return m
  if (ts.isPropertyAssignment(m)) {
    const init = m.initializer
    if (ts.isArrowFunction(init) || ts.isFunctionExpression(init)) return init
  }
  return null
}

function walkForAssignments(
  fn: ts.ArrowFunction | ts.FunctionExpression | ts.MethodDeclaration,
  sf: ts.SourceFile,
  out: Diagnostic[],
): void {
  const visit = (node: ts.Node): void => {
    // Block re-entry into nested arrows/functions — those run in their
    // own context and may legitimately mutate their own locals.
    if (node !== fn && (ts.isArrowFunction(node) || ts.isFunctionExpression(node))) return
    if (
      ts.isBinaryExpression(node) &&
      isAssignmentOp(node.operatorToken.kind) &&
      isMemberAccess(node.left)
    ) {
      out.push({
        range: { start: node.getStart(sf), end: node.getEnd() },
        message: DERIVED_WRITE_MESSAGE,
        severity: 'warning',
        source: 'sui-lsp',
        code: 'derived.mutates-state',
      })
    }
    if (
      (ts.isPostfixUnaryExpression(node) || ts.isPrefixUnaryExpression(node)) &&
      (node.operator === ts.SyntaxKind.PlusPlusToken ||
        node.operator === ts.SyntaxKind.MinusMinusToken) &&
      isMemberAccess(node.operand)
    ) {
      out.push({
        range: { start: node.getStart(sf), end: node.getEnd() },
        message: DERIVED_WRITE_MESSAGE,
        severity: 'warning',
        source: 'sui-lsp',
        code: 'derived.mutates-state',
      })
    }
    ts.forEachChild(node, visit)
  }
  if ('body' in fn && fn.body) ts.forEachChild(fn.body, visit)
}

function isMemberAccess(n: ts.Node): boolean {
  return ts.isPropertyAccessExpression(n) || ts.isElementAccessExpression(n)
}

function isAssignmentOp(k: ts.SyntaxKind): boolean {
  switch (k) {
    case ts.SyntaxKind.EqualsToken:
    case ts.SyntaxKind.PlusEqualsToken:
    case ts.SyntaxKind.MinusEqualsToken:
    case ts.SyntaxKind.AsteriskEqualsToken:
    case ts.SyntaxKind.SlashEqualsToken:
    case ts.SyntaxKind.PercentEqualsToken:
    case ts.SyntaxKind.AmpersandEqualsToken:
    case ts.SyntaxKind.BarEqualsToken:
    case ts.SyntaxKind.CaretEqualsToken:
    case ts.SyntaxKind.LessThanLessThanEqualsToken:
    case ts.SyntaxKind.GreaterThanGreaterThanEqualsToken:
    case ts.SyntaxKind.GreaterThanGreaterThanGreaterThanEqualsToken:
    case ts.SyntaxKind.AmpersandAmpersandEqualsToken:
    case ts.SyntaxKind.BarBarEqualsToken:
    case ts.SyntaxKind.QuestionQuestionEqualsToken:
      return true
    default:
      return false
  }
}

function findDefaultControllerCall(sf: ts.SourceFile): ts.CallExpression | null {
  for (const stmt of sf.statements) {
    if (!ts.isExportAssignment(stmt) || stmt.isExportEquals) continue
    const expr = stmt.expression
    if (
      ts.isCallExpression(expr) &&
      ts.isIdentifier(expr.expression) &&
      expr.expression.text === 'controller'
    ) {
      return expr
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
