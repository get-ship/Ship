// Paired-`.ts` setup-file analysis.
//
// Resolves a `.sui` document's same-stem sibling `.ts`, parses
// it with the TypeScript compiler API, and surfaces the names + types
// returned from the view's default export. The output drives:
//   • template scope resolution (which `{name}` references are valid)
//   • hover (returning the textual type for an identifier)
//   • diagnostics (template references that the setup never returns)

import * as fs from 'node:fs';
import * as path from 'node:path';
import ts from 'typescript';
import { namespacesFromControllerDef, onHandlersFromControllerDef } from './setup-controller';

export interface SetupMember {
  name: string;
  /** Textual type — e.g. `() => number`, `LiveSignal<Foo>`. */
  type: string;
  /** Position of the declaration that produced the type, if known. */
  declarationFile?: string;
  declarationOffset?: number;
}

/** A contribution exposed under a `use:` alias. */
export interface UseNamespaceMember {
  name: string;
  kind: 'state' | 'derived' | 'on' | 'use';
  /** Textual signature — e.g. `() => T[]`, `(s, q: string) => void`. */
  signature: string;
}

/** A `use:` alias and the contributions it exposes. */
export interface UseNamespace {
  alias: string;
  /** Identifier of the hook factory call (`useFilteredList`), or null
   * if the alias was assigned an inline literal. */
  hookFactory: string | null;
  members: UseNamespaceMember[];
}

/** A handler key from the controller's `on:` section. */
export interface OnHandler {
  name: string;
  /** Textual signature — `(s: MutableState<Scope>, e: MouseEvent) => void`. */
  signature: string;
}

export interface SetupInfo {
  uri: string;
  source: string;
  members: SetupMember[];
  /** Positions in the setup file where the *return* object lives — used for
   * surfacing diagnostics that complain about missing keys. */
  returnRange?: { start: number; end: number };
  /** Namespaces contributed via `use:` (D-2). */
  namespaces?: UseNamespace[];
  /** Handlers from the `on:` section (D-4). */
  onHandlers?: OnHandler[];
  /** Template-declared scope members resolved from the sibling `.sui.d.ts`,
   * when the `.ts` imports Scope from a `.sui` path (D-1). */
  scopeMembers?: SetupMember[];
}

export function pairedSetupPath(templateFsPath: string): string | null {
  if (templateFsPath.endsWith('.sui')) {
    return templateFsPath.slice(0, -'.sui'.length) + '.ts';
  }
  return null;
}

/**
 * Given a `.ts` setup file path, find the sibling `.sui.d.ts` file if present.
 * Returns the path or null if not found.
 */
export function pairedDtsPath(setupFsPath: string): string | null {
  const dir = path.dirname(setupFsPath);
  const base = path.basename(setupFsPath);
  const stem = base.endsWith('.ts') ? base.slice(0, -3) : base;
  // stem is e.g. "view" — we look for "view.sui.d.ts" next to "view.ts"
  const candidate = path.join(dir, `${stem}.sui.d.ts`);
  try {
    fs.accessSync(candidate);
    return candidate;
  } catch {
    return null;
  }
}

/**
 * Parse a `.sui.d.ts` file and return the members of its `Scope` interface.
 * Returns an empty array if the file cannot be parsed or has no `Scope`.
 */
export function scopeMembersFromDts(dtsPath: string): SetupMember[] {
  let source: string;
  try {
    source = fs.readFileSync(dtsPath, 'utf8');
  } catch {
    return [];
  }
  return scopeMembersFromDtsSource(dtsPath, source);
}

export function scopeMembersFromDtsSource(dtsPath: string, source: string): SetupMember[] {
  const sf = ts.createSourceFile(
    path.basename(dtsPath), source,
    ts.ScriptTarget.ESNext, true, ts.ScriptKind.TS,
  );
  for (const stmt of sf.statements) {
    if (
      ts.isInterfaceDeclaration(stmt) &&
      ts.isIdentifier(stmt.name) &&
      stmt.name.text === 'Scope'
    ) {
      return stmt.members
        .filter(ts.isPropertySignature)
        .map((m): SetupMember | null => {
          if (!ts.isIdentifier(m.name)) return null;
          const typeTxt = m.type ? m.type.getText(sf) : 'unknown';
          return { name: m.name.text, type: typeTxt };
        })
        .filter((m): m is SetupMember => m !== null);
    }
  }
  return [];
}

export function loadSetup(templateFsPath: string): SetupInfo | null {
  const setupPath = pairedSetupPath(templateFsPath);
  if (!setupPath) return null;
  let source: string;
  try {
    source = fs.readFileSync(setupPath, 'utf8');
  } catch {
    return null;
  }
  return analyzeSetup(setupPath, source);
}

export function analyzeSetup(setupPath: string, source: string): SetupInfo {
  const sf = ts.createSourceFile(
    path.basename(setupPath), source,
    ts.ScriptTarget.ESNext, true, ts.ScriptKind.TS,
  );
  const result: SetupInfo = { uri: setupPath, source, members: [] };

  const def = findDefaultExport(sf);
  if (!def) return result;

  // Form B: extract from the function's return statement.
  let fn: ts.Node | undefined;
  if (ts.isCallExpression(def)) fn = def.arguments[0];
  else if (ts.isFunctionDeclaration(def) || ts.isArrowFunction(def) || ts.isFunctionExpression(def)) fn = def;
  if (fn && (ts.isArrowFunction(fn) || ts.isFunctionExpression(fn) || ts.isFunctionDeclaration(fn))) {
    const ret = findFinalReturn(fn);
    if (ret) {
      result.returnRange = { start: ret.getStart(sf), end: ret.getEnd() };
      if (ret.expression && ts.isObjectLiteralExpression(ret.expression)) {
        result.members = membersFromObjectLiteral(ret.expression);
      }
    }
  }

  // Form C: `controller(def)` from `@ship/ui-runtime`. The wrapped
  // `def` is either an object literal (tier 2) or an arrow returning
  // one (tier 3). Walk its `state` / `derived` / `on` / `use` sections
  // and emit the section keys as scope members. Override Form B's
  // result if it found anything — controller's section-walk is more
  // accurate than a generic return-object extraction would be.
  if (
    isControllerCall(def)
  ) {
    const arg = def.arguments[0]!;
    const obj = unwrapToObjectLiteral(arg)
    if (obj) {
      const members = membersFromControllerDef(obj)
      if (members.length > 0) {
        result.members = members
        result.returnRange = { start: obj.getStart(sf), end: obj.getEnd() }
      }
      result.namespaces = namespacesFromControllerDef(obj, sf)
      result.onHandlers = onHandlersFromControllerDef(obj, sf)
    }
  }

  // D-1 enhancement: if the .ts file imports `Scope` from a `.sui` path,
  // resolve the sibling `.sui.d.ts` and enrich `scopeMembers` with typed
  // declarations. These are the template-declared bindings (state, data,
  // props, use-* hooks) the controller may not expose in its own sections.
  const dtsPath = pairedDtsPath(setupPath);
  if (dtsPath) {
    const dtsMembers = scopeMembersFromDts(dtsPath);
    if (dtsMembers.length > 0) {
      // Merge: prefer dts types for names already in members (they're more
      // specific), and add dts-only names (state/data declared in template).
      const byName = new Map(result.members.map((m) => [m.name, m]));
      for (const dm of dtsMembers) {
        const existing = byName.get(dm.name);
        if (existing) {
          existing.type = dm.type;
        } else {
          byName.set(dm.name, dm);
        }
      }
      result.members = [...byName.values()];
      result.scopeMembers = dtsMembers;
    }
  }

  return result;
}

function isControllerCall(def: ts.Node): def is ts.CallExpression {
  if (!ts.isCallExpression(def)) return false
  if (def.arguments.length === 0) return false
  // `controller(def)` or `controller<Scope>(def)`. Both have an
  // `Identifier` callee named `controller`.
  return ts.isIdentifier(def.expression) && def.expression.text === 'controller'
}

/** Walk `expr` until we land on an object literal:
 * - `{...}` → the literal itself
 * - `(...) => ({...})` → the arrow's expression body
 * - `(...) => { ... return {...} }` → the final return statement
 * - anything else → null
 */
function unwrapToObjectLiteral(expr: ts.Node): ts.ObjectLiteralExpression | null {
  if (ts.isObjectLiteralExpression(expr)) return expr
  if (ts.isParenthesizedExpression(expr)) return unwrapToObjectLiteral(expr.expression)
  if (ts.isArrowFunction(expr) || ts.isFunctionExpression(expr)) {
    if (!ts.isBlock(expr.body)) {
      // arrow-expression body
      return unwrapToObjectLiteral(expr.body)
    }
    const ret = findFinalReturn(expr)
    if (ret && ret.expression) return unwrapToObjectLiteral(ret.expression)
  }
  return null
}

/** Walk a `controller(def)` argument object literal. The relevant
 * sections each contribute identifiers to the view scope:
 *
 *   - `derived`: keys are callable memos
 *   - `on`:      keys are handler functions
 *   - `use`:     keys are hook namespace objects
 *
 * Per the "state is declarative-only" ADR, `state:` is no longer a
 * controller section — local writable signals are declared in the
 * template via `<state name="..." />` and reach scope through that
 * route. The LSP picks them up via `parsed.state` from the template
 * parser, not from the controller def.
 *
 * Each section may itself be an object literal OR a factory `(scope) => ({...})`.
 * Both are unwrapped via `unwrapToObjectLiteral`.
 */
function membersFromControllerDef(obj: ts.ObjectLiteralExpression): SetupMember[] {
  const out: SetupMember[] = []
  for (const sectionName of ['derived', 'on', 'use'] as const) {
    const prop = obj.properties.find(
      (p) => ts.isPropertyAssignment(p) && ts.isIdentifier(p.name) && p.name.text === sectionName,
    )
    if (!prop || !ts.isPropertyAssignment(prop)) continue
    const innerObj = unwrapToObjectLiteral(prop.initializer)
    if (!innerObj) continue
    for (const member of innerObj.properties) {
      if (
        (ts.isPropertyAssignment(member) || ts.isShorthandPropertyAssignment(member) || ts.isMethodDeclaration(member)) &&
        member.name &&
        ts.isIdentifier(member.name)
      ) {
        out.push({ name: member.name.text, type: 'unknown' })
      }
    }
  }
  return out
}

function findDefaultExport(sf: ts.SourceFile): ts.Expression | ts.FunctionDeclaration | null {
  for (const stmt of sf.statements) {
    if (ts.isExportAssignment(stmt) && !stmt.isExportEquals) return stmt.expression;
    if (ts.isFunctionDeclaration(stmt) && hasDefaultExportModifier(stmt)) return stmt;
  }
  return null;
}

function hasDefaultExportModifier(node: ts.HasModifiers): boolean {
  const mods = ts.canHaveModifiers(node) ? ts.getModifiers(node) : undefined;
  if (!mods) return false;
  let hasExport = false;
  let hasDefault = false;
  for (const m of mods) {
    if (m.kind === ts.SyntaxKind.ExportKeyword) hasExport = true;
    if (m.kind === ts.SyntaxKind.DefaultKeyword) hasDefault = true;
  }
  return hasExport && hasDefault;
}

function findFinalReturn(
  fn: ts.ArrowFunction | ts.FunctionExpression | ts.FunctionDeclaration,
): ts.ReturnStatement | null {
  if (ts.isArrowFunction(fn) && !ts.isBlock(fn.body)) {
    return null;
  }
  const body = fn.body as ts.Block | undefined;
  if (!body) return null;
  for (let i = body.statements.length - 1; i >= 0; i--) {
    const s = body.statements[i]!;
    if (ts.isReturnStatement(s)) return s;
  }
  return null;
}

function membersFromObjectLiteral(o: ts.ObjectLiteralExpression): SetupMember[] {
  const out: SetupMember[] = [];
  for (const p of o.properties) {
    if (ts.isShorthandPropertyAssignment(p) && ts.isIdentifier(p.name)) {
      out.push({ name: p.name.text, type: 'unknown' });
    } else if (ts.isPropertyAssignment(p) && ts.isIdentifier(p.name)) {
      out.push({ name: p.name.text, type: 'unknown' });
    } else if (ts.isMethodDeclaration(p) && ts.isIdentifier(p.name)) {
      out.push({ name: p.name.text, type: 'function' });
    }
  }
  return out;
}
