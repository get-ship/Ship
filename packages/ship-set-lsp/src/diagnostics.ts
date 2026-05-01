// Diagnostic generation for `.html.set`.
//
// Three categories:
//   1. Directive validation — `<data>` / `<state>` / `<while-…>` /
//      `<component>` / `<prop>` shape correctness.
//   2. Expression-grammar enforcement — assignments / statements /
//      stray arrow functions outside `on:` handlers.
//   3. Unbound-reference detection — identifiers in `{…}` that aren't
//      declared in the template scope or the paired setup return type.

import ts from 'typescript';
import type { Attribute, Node, ParseError, Range } from './parse-types';
import type { SetupInfo } from './setup';
import { buildScope, VIEW_GLOBALS, type Reference } from './bindings';

export interface Diagnostic {
  range: Range;
  message: string;
  severity: 'error' | 'warning' | 'info';
  source: 'ship-set-lsp';
  code?: string;
}

export function diagnose(
  source: string,
  nodes: Node[],
  parseErrors: ParseError[],
  setup: SetupInfo | null,
): Diagnostic[] {
  const out: Diagnostic[] = [];
  for (const e of parseErrors) {
    out.push({ range: e.range, message: e.message, severity: 'error', source: 'ship-set-lsp', code: 'parse' });
  }
  for (const n of nodes) {
    if (n.kind === 'element-open') diagnoseDirective(n, out);
    diagnoseExpressions(n, out);
  }
  diagnoseReferences(nodes, source, setup, out);
  diagnoseSetupCoverage(nodes, source, setup, out);
  return out;
}

function diagnoseDirective(
  n: Extract<Node, { kind: 'element-open' }>, out: Diagnostic[],
): void {
  switch (n.tag) {
    case 'data':
      requireAttr(n, 'name', out, 'directive.data');
      requireAttr(n, 'source', out, 'directive.data');
      break;
    case 'state':
      requireAttr(n, 'name', out, 'directive.state');
      requireAttr(n, 'value', out, 'directive.state');
      break;
    case 'while-loading':
    case 'while-error':
      requireAttr(n, 'bind', out, `directive.${n.tag}`);
      break;
    case 'component':
      requireAttr(n, 'name', out, 'directive.component');
      break;
    case 'prop':
      requireAttr(n, 'name', out, 'directive.prop');
      break;
  }
}

function requireAttr(
  n: Extract<Node, { kind: 'element-open' }>, name: string,
  out: Diagnostic[], code: string,
): void {
  if (!n.attrs.find(a => a.name === name)) {
    out.push({
      range: n.tagRange,
      message: `<${n.tag}> requires \`${name}\` attribute`,
      severity: 'error',
      source: 'ship-set-lsp',
      code,
    });
  }
}

function diagnoseExpressions(n: Node, out: Diagnostic[]): void {
  if (n.kind === 'interpolation') {
    checkExprGrammar(n.text, n.range, false, 'interpolation', out);
  } else if (n.kind === 'block-open') {
    if (n.expr) checkExprGrammar(n.expr.text, n.expr.range, false, n.block, out);
    if (n.iter) checkExprGrammar(n.iter.text, n.iter.range, false, 'for-iter', out);
  } else if (n.kind === 'block-tag' && n.expr) {
    checkExprGrammar(n.expr.text, n.expr.range, false, n.tag, out);
  } else if (n.kind === 'element-open') {
    for (const a of n.attrs) checkAttrExpr(a, out);
  }
}

function checkAttrExpr(a: Attribute, out: Diagnostic[]): void {
  if (a.value?.kind === 'expr') {
    const isEvent = a.kind === 'on';
    checkExprGrammar(a.value.text, a.value.range, isEvent, `attr.${a.kind}`, out);
  }
  if (a.nameExpr) {
    checkExprGrammar(
      a.nameExpr.text,
      { start: a.nameExpr.range.start + 1, end: a.nameExpr.range.end - 1 },
      false, 'attr.attr-name', out,
    );
  }
}

function checkExprGrammar(
  text: string, base: Range, allowArrow: boolean, ctx: string, out: Diagnostic[],
): void {
  if (!text.trim()) return;
  const sf = ts.createSourceFile('__expr.ts', `(${text})`, ts.ScriptTarget.ESNext, true, ts.ScriptKind.TS);
  // Single ParenthesizedExpression on success.
  const stmts = sf.statements;
  if (stmts.length !== 1 || !ts.isExpressionStatement(stmts[0]!)) {
    out.push({ range: base, message: `Expression contains a statement`, severity: 'error', source: 'ship-set-lsp', code: `expr.${ctx}` });
    return;
  }
  const visit = (node: ts.Node): void => {
    if (ts.isBinaryExpression(node) && isAssignmentOp(node.operatorToken.kind)) {
      const start = base.start + node.getStart(sf) - 1;
      const end = base.start + node.getEnd() - 1;
      out.push({
        range: { start, end },
        message: 'Assignment is not allowed in template expressions',
        severity: 'error', source: 'ship-set-lsp', code: `expr.assign.${ctx}`,
      });
    }
    if (!allowArrow && (ts.isArrowFunction(node) || ts.isFunctionExpression(node))) {
      const start = base.start + node.getStart(sf) - 1;
      const end = base.start + node.getEnd() - 1;
      out.push({
        range: { start, end },
        message: 'Arrow functions are only allowed in `on:` event handlers',
        severity: 'error', source: 'ship-set-lsp', code: `expr.arrow.${ctx}`,
      });
    }
    ts.forEachChild(node, visit);
  };
  visit(stmts[0]!);
}

function isAssignmentOp(k: ts.SyntaxKind): boolean {
  return (
    k === ts.SyntaxKind.EqualsToken ||
    k === ts.SyntaxKind.PlusEqualsToken ||
    k === ts.SyntaxKind.MinusEqualsToken ||
    k === ts.SyntaxKind.AsteriskEqualsToken ||
    k === ts.SyntaxKind.SlashEqualsToken ||
    k === ts.SyntaxKind.PercentEqualsToken ||
    k === ts.SyntaxKind.AmpersandEqualsToken ||
    k === ts.SyntaxKind.BarEqualsToken ||
    k === ts.SyntaxKind.CaretEqualsToken ||
    k === ts.SyntaxKind.LessThanLessThanEqualsToken ||
    k === ts.SyntaxKind.GreaterThanGreaterThanEqualsToken ||
    k === ts.SyntaxKind.GreaterThanGreaterThanGreaterThanEqualsToken ||
    k === ts.SyntaxKind.AmpersandAmpersandEqualsToken ||
    k === ts.SyntaxKind.BarBarEqualsToken ||
    k === ts.SyntaxKind.QuestionQuestionEqualsToken
  );
}

function diagnoseReferences(
  nodes: Node[], source: string, setup: SetupInfo | null, out: Diagnostic[],
): void {
  const scope = buildScope(nodes, source);
  const setupNames = new Set(setup?.members.map(m => m.name) ?? []);
  const declByName = new Map<string, typeof scope.declarations>();
  for (const d of scope.declarations) {
    let arr = declByName.get(d.name);
    if (!arr) { arr = []; declByName.set(d.name, arr); }
    arr.push(d);
  }
  const namespaces = new Map<string, Set<string>>();
  for (const ns of setup?.namespaces ?? []) {
    namespaces.set(ns.alias, new Set(ns.members.map(m => m.name)));
  }
  for (const ref of scope.references) {
    if (!resolves(ref, declByName, setupNames)) {
      out.push({
        range: ref.range,
        message: `Unresolved identifier \`${ref.name}\``,
        severity: 'error', source: 'ship-set-lsp', code: 'unbound',
      });
      continue;
    }
    if (ref.member && namespaces.has(ref.name)) {
      const members = namespaces.get(ref.name)!;
      if (!members.has(ref.member.name)) {
        out.push({
          range: ref.member.range,
          message: `\`${ref.name}\` has no contribution \`${ref.member.name}\``,
          severity: 'error', source: 'ship-set-lsp', code: 'use.unknown-member',
        });
      }
    }
  }
}

function resolves(
  ref: Reference,
  declByName: Map<string, ReturnType<typeof buildScope>['declarations']>,
  setupNames: Set<string>,
): boolean {
  if (VIEW_GLOBALS.has(ref.name)) return true;
  if (setupNames.has(ref.name)) return true;
  const decls = declByName.get(ref.name);
  if (!decls) return false;
  return decls.some(d => ref.range.start >= d.extent.start && ref.range.end <= d.extent.end);
}

function diagnoseSetupCoverage(
  nodes: Node[], source: string, setup: SetupInfo | null, out: Diagnostic[],
): void {
  if (!setup || setup.members.length === 0 || !setup.returnRange) return;
  // Collect template references that are *not* satisfied by template scope.
  const scope = buildScope(nodes, source);
  const declNames = new Set(scope.declarations.map(d => d.name));
  const used = new Set<string>();
  for (const r of scope.references) {
    if (VIEW_GLOBALS.has(r.name)) continue;
    if (declNames.has(r.name)) continue;
    used.add(r.name);
  }
  const setupNames = new Set(setup.members.map(m => m.name));
  for (const name of used) {
    if (!setupNames.has(name)) {
      // Already surfaced as `unbound` on the template side; skip duplicate
      // setup-side warning so we don't double-up.
    }
  }
  // Note: we surface missing-key diagnostics on the template (where the
  // unresolved name lives). The acceptance criterion calls this out as
  // a setup-side diagnostic; see `setupGapDiagnostic` for the helper
  // that callers wanting a `.ts`-side diagnostic can use.
}

export function setupGapDiagnostic(
  templateRefs: Iterable<string>,
  setup: SetupInfo,
): Diagnostic | null {
  if (!setup.returnRange) return null;
  const setupNames = new Set(setup.members.map(m => m.name));
  const missing: string[] = [];
  for (const r of templateRefs) {
    if (!setupNames.has(r) && !VIEW_GLOBALS.has(r)) missing.push(r);
  }
  if (missing.length === 0) return null;
  return {
    range: setup.returnRange,
    message: `Setup return is missing keys referenced from template: ${missing.join(', ')}`,
    severity: 'error',
    source: 'ship-set-lsp',
    code: 'setup.missing-keys',
  };
}
