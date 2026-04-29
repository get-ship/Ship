// Paired-`.ts` setup-file analysis.
//
// Resolves a `.html.set` document's same-stem sibling `.ts`, parses
// it with the TypeScript compiler API, and surfaces the names + types
// returned from the view's default export. The output drives:
//   • template scope resolution (which `{name}` references are valid)
//   • hover (returning the textual type for an identifier)
//   • diagnostics (template references that the setup never returns)

import * as fs from 'node:fs';
import * as path from 'node:path';
import ts from 'typescript';

export interface SetupMember {
  name: string;
  /** Textual type — e.g. `() => number`, `LiveSignal<Foo>`. */
  type: string;
  /** Position of the declaration that produced the type, if known. */
  declarationFile?: string;
  declarationOffset?: number;
}

export interface SetupInfo {
  uri: string;
  source: string;
  members: SetupMember[];
  /** Positions in the setup file where the *return* object lives — used for
   * surfacing diagnostics that complain about missing keys. */
  returnRange?: { start: number; end: number };
}

export function pairedSetupPath(templateFsPath: string): string | null {
  if (templateFsPath.endsWith('.html.set')) {
    return templateFsPath.slice(0, -'.html.set'.length) + '.ts';
  }
  if (templateFsPath.endsWith('.set')) {
    return templateFsPath.slice(0, -'.set'.length) + '.ts';
  }
  return null;
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

  // Form A: export default view<Bindings>(fn) — extract from generic arg.
  if (ts.isCallExpression(def) && def.typeArguments && def.typeArguments.length > 0) {
    const typeArg = def.typeArguments[0]!;
    const members = membersFromTypeNode(sf, typeArg);
    if (members.length > 0) {
      result.members = members;
      const fnArg = def.arguments[0];
      if (fnArg && (ts.isArrowFunction(fnArg) || ts.isFunctionExpression(fnArg))) {
        const ret = findFinalReturn(fnArg);
        if (ret) result.returnRange = { start: ret.getStart(sf), end: ret.getEnd() };
      }
      return result;
    }
  }

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
  return result;
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

function membersFromTypeNode(sf: ts.SourceFile, t: ts.TypeNode): SetupMember[] {
  if (ts.isTypeLiteralNode(t)) return membersFromTypeLiteral(t);
  if (ts.isTypeReferenceNode(t) && ts.isIdentifier(t.typeName)) {
    const name = t.typeName.text;
    const decl = findInterfaceOrAlias(sf, name);
    if (!decl) return [];
    if (ts.isInterfaceDeclaration(decl)) {
      return decl.members.flatMap(m => memberFromSignatureMember(m));
    }
    if (ts.isTypeAliasDeclaration(decl) && ts.isTypeLiteralNode(decl.type)) {
      return membersFromTypeLiteral(decl.type);
    }
  }
  return [];
}

function findInterfaceOrAlias(
  sf: ts.SourceFile, name: string,
): ts.InterfaceDeclaration | ts.TypeAliasDeclaration | null {
  for (const s of sf.statements) {
    if (ts.isInterfaceDeclaration(s) && s.name.text === name) return s;
    if (ts.isTypeAliasDeclaration(s) && s.name.text === name) return s;
  }
  return null;
}

function membersFromTypeLiteral(t: ts.TypeLiteralNode): SetupMember[] {
  return t.members.flatMap(m => memberFromSignatureMember(m));
}

function memberFromSignatureMember(m: ts.TypeElement): SetupMember[] {
  if (ts.isPropertySignature(m) && m.name && ts.isIdentifier(m.name)) {
    const t = m.type ? m.type.getText() : 'unknown';
    return [{ name: m.name.text, type: t }];
  }
  if (ts.isMethodSignature(m) && m.name && ts.isIdentifier(m.name)) {
    const params = m.parameters.map(p => p.getText()).join(', ');
    const ret = m.type ? m.type.getText() : 'void';
    return [{ name: m.name.text, type: `(${params}) => ${ret}` }];
  }
  return [];
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
