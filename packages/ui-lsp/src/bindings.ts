// Template scope extraction.
//
// Walks a parsed `.sui` document and produces:
//   • declarations: identifiers declared by `<data>` / `<state>` /
//     `<prop>` / `let:` / `{#for}` patterns
//   • references: identifiers used inside `{…}` expressions (and
//     binding-attribute expressions) along with the surrounding scope
//
// The output drives diagnostics (unbound-reference detection) and
// completion (in-scope identifier suggestions).

import ts from 'typescript';
import type { Attribute, Node, Range } from './parse-types';

export type DeclSource = 'data' | 'state' | 'prop' | 'let' | 'for';

export interface Declaration {
  name: string;
  source: DeclSource;
  // Range that introduced the declaration (the `name="..."` attr value, etc.)
  range: Range;
  // Lexical extent — references inside this range can resolve to it.
  // For top-level declarations (data/state/prop), extent covers the
  // whole document.
  extent: Range;
}

export interface Reference {
  name: string;
  range: Range;
  // Surrounding expression text (for context display)
  expr: string;
  exprRange: Range;
  // 'on' handlers permit arrow functions; other contexts don't.
  inEventHandler: boolean;
  // When the bare identifier is the LHS of a property access — e.g.
  // `list.filtered` — this captures the accessed member's name and
  // source range. Drives `use:` namespace member validation (D-2).
  member?: { name: string; range: Range };
}

export interface Scope {
  declarations: Declaration[];
  references: Reference[];
}

// JS / view globals that always resolve.
export const VIEW_GLOBALS = new Set<string>([
  'route', 'ship', 'navigate',
  'true', 'false', 'null', 'undefined', 'NaN', 'Infinity',
  'Math', 'Object', 'Array', 'Number', 'String', 'Boolean', 'JSON',
  'console', 'Date', 'Promise', 'Error', 'Map', 'Set', 'Symbol',
  'parseInt', 'parseFloat', 'isNaN', 'isFinite',
]);

export function buildScope(nodes: Node[], source: string): Scope {
  const decls: Declaration[] = [];
  const refs: Reference[] = [];
  const docExtent: Range = { start: 0, end: source.length };

  // Stack of for-loop scopes: { idents, extent }
  const forStack: { idents: Declaration[] }[] = [];
  // Stack of let:/while-* element scopes: keyed by element-open index.
  // We don't track element-close pairing strictly; for Phase 5,
  // `let:` declarations are visible within the element's open-tag-end
  // through a heuristic: scan ahead to a matching close tag.

  for (let i = 0; i < nodes.length; i++) {
    const n = nodes[i]!;
    if (n.kind === 'element-open') {
      collectFromElement(n, nodes, i, decls, source, docExtent);
    } else if (n.kind === 'block-open' && n.block === 'for' && n.pattern) {
      const extent = forBlockExtent(nodes, i, n.pattern.range.end);
      const idents: Declaration[] = n.pattern.idents.map(name => ({
        name, source: 'for', range: n.pattern!.range, extent,
      }));
      decls.push(...idents);
      forStack.push({ idents });
    } else if (n.kind === 'block-close' && n.block === 'for') {
      forStack.pop();
    }
    // Reference extraction
    collectExpressionsAt(n, refs);
  }
  return { declarations: decls, references: refs };
}

function collectFromElement(
  n: Extract<Node, { kind: 'element-open' }>,
  nodes: Node[], idx: number,
  decls: Declaration[], source: string, docExtent: Range,
): void {
  if (n.tag === 'data' || n.tag === 'state' || n.tag === 'prop') {
    const nameAttr = n.attrs.find(a => a.name === 'name' && a.value?.kind === 'string');
    if (nameAttr?.value) {
      decls.push({
        name: nameAttr.value.text,
        source: n.tag,
        range: nameAttr.value.range,
        extent: docExtent,
      });
    }
  }
  // let:foo on any element — visible inside that element's children
  for (const a of n.attrs) {
    if (a.kind !== 'let') continue;
    const extent = elementInnerExtent(nodes, idx);
    decls.push({ name: a.name, source: 'let', range: a.nameRange, extent });
  }
}

function elementInnerExtent(nodes: Node[], openIdx: number): Range {
  const open = nodes[openIdx]! as Extract<Node, { kind: 'element-open' }>;
  const start = open.range.end;
  if (open.selfClosing) return { start, end: start };
  let depth = 0;
  for (let j = openIdx + 1; j < nodes.length; j++) {
    const m = nodes[j]!;
    if (m.kind === 'element-open' && !m.selfClosing && m.tag === open.tag) depth++;
    else if (m.kind === 'element-close' && m.tag === open.tag) {
      if (depth === 0) return { start, end: m.range.start };
      depth--;
    }
  }
  return { start, end: Number.MAX_SAFE_INTEGER };
}

function forBlockExtent(nodes: Node[], openIdx: number, start: number): Range {
  let depth = 0;
  for (let j = openIdx + 1; j < nodes.length; j++) {
    const m = nodes[j]!;
    if (m.kind === 'block-open' && m.block === 'for') depth++;
    else if (m.kind === 'block-close' && m.block === 'for') {
      if (depth === 0) return { start, end: m.outerRange.start };
      depth--;
    }
  }
  return { start, end: Number.MAX_SAFE_INTEGER };
}

function collectExpressionsAt(n: Node, refs: Reference[]): void {
  const exprs: { text: string; range: Range; isEvent: boolean }[] = [];
  if (n.kind === 'interpolation') {
    exprs.push({ text: n.text, range: n.range, isEvent: false });
  } else if (n.kind === 'block-open') {
    if (n.expr) exprs.push({ text: n.expr.text, range: n.expr.range, isEvent: false });
    if (n.iter) exprs.push({ text: n.iter.text, range: n.iter.range, isEvent: false });
    if (n.key) {
      exprs.push({
        text: n.key.text,
        range: { start: n.key.range.start + 1, end: n.key.range.end - 1 },
        isEvent: false,
      });
    }
  } else if (n.kind === 'block-tag' && n.expr) {
    exprs.push({ text: n.expr.text, range: n.expr.range, isEvent: false });
  } else if (n.kind === 'element-open') {
    for (const a of n.attrs) collectFromAttribute(a, exprs);
  }
  for (const e of exprs) extractIdentifiers(e.text, e.range, e.isEvent, refs);
}

function collectFromAttribute(
  a: Attribute,
  out: { text: string; range: Range; isEvent: boolean }[],
): void {
  if (a.value?.kind === 'expr') {
    out.push({ text: a.value.text, range: a.value.range, isEvent: a.kind === 'on' });
  }
  if (a.nameExpr) {
    out.push({
      text: a.nameExpr.text,
      range: { start: a.nameExpr.range.start + 1, end: a.nameExpr.range.end - 1 },
      isEvent: false,
    });
  }
  // bind:value="agents" — `agents` after bind: is a reference to the local scope
  if (a.kind === 'bind' && !a.value && a.name) {
    // bind without `=` form not valid; skip
  }
  // <while-loading bind="agents"> — bind is a *static* attr name with string value.
  // Detect later in the diagnostics layer; here we handle binding-attribute exprs only.
}

function extractIdentifiers(
  text: string, baseRange: Range, isEvent: boolean, refs: Reference[],
): void {
  if (!text.trim()) return;
  let sf: ts.SourceFile;
  try {
    sf = ts.createSourceFile('__expr.ts', `(${text})`, ts.ScriptTarget.ESNext, true, ts.ScriptKind.TS);
  } catch {
    return;
  }
  // The wrapping `(` at offset 0 shifts AST positions by +1. Subtract 1.
  const localScope = new Set<string>();
  const visit = (node: ts.Node, scopes: Set<string>[]) => {
    if (ts.isArrowFunction(node) || ts.isFunctionExpression(node)) {
      const innerScope = new Set<string>(scopes[scopes.length - 1]);
      for (const p of node.parameters) {
        for (const name of bindingNames(p.name)) innerScope.add(name);
      }
      ts.forEachChild(node, c => visit(c, [...scopes, innerScope]));
      return;
    }
    if (ts.isVariableDeclaration(node)) {
      for (const name of bindingNames(node.name)) scopes[scopes.length - 1]!.add(name);
    }
    if (ts.isIdentifier(node) && !isPropertyAccessName(node) && !isJsxName(node)) {
      const inScope = scopes.some(s => s.has(node.text));
      if (!inScope) {
        const start = baseRange.start + node.getStart(sf) - 1;
        const end = baseRange.start + node.getEnd() - 1;
        const member = memberAccessOf(node, baseRange.start);
        refs.push({
          name: node.text,
          range: { start, end },
          expr: text,
          exprRange: baseRange,
          inEventHandler: isEvent,
          ...(member ? { member } : {}),
        });
      }
    }
    ts.forEachChild(node, c => visit(c, scopes));
  };
  visit(sf, [localScope]);
}

function bindingNames(name: ts.BindingName): string[] {
  if (ts.isIdentifier(name)) return [name.text];
  const out: string[] = [];
  if (ts.isObjectBindingPattern(name) || ts.isArrayBindingPattern(name)) {
    for (const e of name.elements) {
      if (ts.isBindingElement(e)) out.push(...bindingNames(e.name));
    }
  }
  return out;
}

function memberAccessOf(
  id: ts.Identifier, baseStart: number,
): { name: string; range: Range } | null {
  const p = id.parent;
  if (!p || !ts.isPropertyAccessExpression(p)) return null;
  if (p.expression !== id) return null;
  const m = p.name;
  if (!ts.isIdentifier(m)) return null;
  const sf = id.getSourceFile();
  return {
    name: m.text,
    range: {
      start: baseStart + m.getStart(sf) - 1,
      end: baseStart + m.getEnd() - 1,
    },
  };
}

function isPropertyAccessName(id: ts.Identifier): boolean {
  const p = id.parent;
  if (!p) return false;
  if (ts.isPropertyAccessExpression(p) && p.name === id) return true;
  if (ts.isQualifiedName(p) && p.right === id) return true;
  if (ts.isPropertyAssignment(p) && p.name === id) return true;
  if (ts.isShorthandPropertyAssignment(p)) return false;
  return false;
}

function isJsxName(_id: ts.Identifier): boolean {
  return false;
}
