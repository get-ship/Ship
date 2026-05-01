// Setup-file diagnostics emitted alongside `.html.set` analysis.
//
// Today's only producer is the derived-write check (D-3): `derived`
// sections receive a readonly view of state — assignments inside them
// are bugs. The TypeScript type-checker rejects these via the
// `ReadonlyState<…>` parameter, but only when the user runs
// `tsc --noEmit`. This module surfaces the same finding directly so
// the LSP can publish a Warning marker on the offending line as soon
// as the file is parsed.
//
// Output is the LSP's standard `Diagnostic` shape so the server can
// forward it under the setup file's URI without further translation.

import ts from 'typescript'
import * as path from 'node:path'
import type { Diagnostic } from './diagnostics'

const DERIVED_WRITE_MESSAGE =
  "derived sections cannot mutate state — move the assignment to an 'on' handler or 'mount'"

export function diagnoseSetup(setupPath: string, source: string): Diagnostic[] {
  const sf = ts.createSourceFile(
    path.basename(setupPath),
    source,
    ts.ScriptTarget.ESNext,
    true,
    ts.ScriptKind.TS,
  )
  const out: Diagnostic[] = []
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
        source: 'ship-set-lsp',
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
        source: 'ship-set-lsp',
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
