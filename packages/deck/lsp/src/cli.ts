#!/usr/bin/env bun
// One-shot CLI for the sui-lsp analyzers.
//
// Mirrors the LSP server's analysis path but emits JSON to stdout so MCP
// tools (and other agents) can consume diagnostics + hover without
// embedding an LSP client.
//
// Usage:
//   bun packages/deck/lsp/src/cli.ts diagnostics <file.sui>
//   bun packages/deck/lsp/src/cli.ts hover <file.sui> --line <n> --col <n>
//   bun packages/deck/lsp/src/cli.ts dependencies <file.sui>

import * as fs from 'node:fs';
import * as path from 'node:path';
import * as url from 'node:url';
import { parse, type Node } from './parse';
import { diagnose } from './diagnostics';
import { hover } from './hover';
import { analyzeSetup, loadSetup, type SetupInfo } from './setup';

interface ParsedArgs {
  subcommand: string;
  file: string;
  line?: number;
  col?: number;
}

function parseArgs(argv: string[]): ParsedArgs {
  const subcommand = argv[0];
  if (!subcommand) {
    throw new Error(
      'usage: diagnostics <file> | hover <file> --line N --col N | dependencies <file>',
    );
  }
  const file = argv[1];
  if (!file) throw new Error('missing <file> argument');
  let line: number | undefined;
  let col: number | undefined;
  for (let i = 2; i < argv.length; i++) {
    const a = argv[i];
    if (a === '--line') line = Number(argv[++i]);
    else if (a === '--col') col = Number(argv[++i]);
    else throw new Error(`unknown arg: ${a}`);
  }
  return { subcommand, file, line, col };
}

function offsetAt(source: string, line: number, col: number): number {
  let off = 0;
  let curLine = 0;
  while (curLine < line) {
    const nl = source.indexOf('\n', off);
    if (nl === -1) return source.length;
    off = nl + 1;
    curLine++;
  }
  return Math.min(off + col, source.length);
}

function positionAt(source: string, offset: number): { line: number; col: number } {
  let line = 0;
  let lastNl = -1;
  for (let i = 0; i < offset && i < source.length; i++) {
    if (source[i] === '\n') {
      line++;
      lastNl = i;
    }
  }
  return { line, col: offset - lastNl - 1 };
}

function runDiagnostics(file: string): void {
  if (!fs.existsSync(file)) {
    process.stderr.write(JSON.stringify({ error: `file not found: ${file}` }) + '\n');
    process.exit(2);
  }
  const source = fs.readFileSync(file, 'utf8');
  const parsed = parse(source);
  const setup = resolveSetup(file);
  const diagnostics = diagnose(source, parsed.nodes, parsed.errors, setup);
  const out = diagnostics.map(d => ({
    range: {
      start: positionAt(source, d.range.start),
      end: positionAt(source, d.range.end),
    },
    severity: d.severity,
    code: d.code ?? null,
    source: d.source,
    message: d.message,
  }));
  process.stdout.write(JSON.stringify({ diagnostics: out }) + '\n');
}

function runHover(file: string, line: number, col: number): void {
  if (!fs.existsSync(file)) {
    process.stderr.write(JSON.stringify({ error: `file not found: ${file}` }) + '\n');
    process.exit(2);
  }
  const source = fs.readFileSync(file, 'utf8');
  const parsed = parse(source);
  const setup = resolveSetup(file);
  const offset = offsetAt(source, line, col);
  const h = hover(source, offset, parsed.nodes, setup);
  if (!h) {
    process.stdout.write(JSON.stringify({ contents: null, range: null }) + '\n');
    return;
  }
  process.stdout.write(JSON.stringify({
    contents: h.contents,
    range: {
      start: positionAt(source, h.range.start),
      end: positionAt(source, h.range.end),
    },
  }) + '\n');
}

// ── dependencies ──────────────────────────────────────────────────────
//
// Surface the SUI surface a `.sui` view depends on: custom-element tags
// (sui-*, ship-*, third-party), event/binding directives, and identifier
// references partitioned against the paired controller's setup. Lets
// agents answer "what does this view touch?" without re-parsing.

const IDENT_RE = /[A-Za-z_$][A-Za-z0-9_$]*/g;
const SKIP_IDENTS = new Set([
  'true', 'false', 'null', 'undefined', 'this',
  'item', 'index', 'i', 'event', 'e', 'value',
  'String', 'Number', 'Boolean', 'Array', 'Object', 'Math', 'JSON', 'Date',
]);

interface DepReport {
  file: string;
  tags: string[];
  signals: string[];        // setup-backed identifiers referenced by the view
  scope_unknown: string[];  // identifiers referenced but not in setup
  events: string[];         // on:* event names
  bindings: string[];       // bind:* target names
  namespaces: string[];     // use:* aliases referenced
  controller: string | null;
}

function runDependencies(file: string): void {
  if (!fs.existsSync(file)) {
    process.stderr.write(JSON.stringify({ error: `file not found: ${file}` }) + '\n');
    process.exit(2);
  }
  const source = fs.readFileSync(file, 'utf8');
  const parsed = parse(source);
  const setup = resolveSetup(file);
  const report = collectDependencies(file, parsed.nodes, setup);
  process.stdout.write(JSON.stringify(report) + '\n');
}

function collectDependencies(
  file: string, nodes: Node[], setup: SetupInfo | null,
): DepReport {
  const tags = new Set<string>();
  const events = new Set<string>();
  const bindings = new Set<string>();
  const exprTexts: string[] = [];
  for (const n of nodes) {
    if (n.kind === 'element-open') {
      if (n.tag.includes('-')) tags.add(n.tag);
      for (const a of n.attrs) {
        if (a.kind === 'on') events.add(a.name);
        if (a.kind === 'bind') bindings.add(a.name);
        if (a.value?.kind === 'expr') exprTexts.push(a.value.text);
        if (a.nameExpr) exprTexts.push(a.nameExpr.text);
      }
    } else if (n.kind === 'interpolation') {
      exprTexts.push(n.text);
    } else if (n.kind === 'block-open') {
      if (n.expr) exprTexts.push(n.expr.text);
      if (n.iter) exprTexts.push(n.iter.text);
      if (n.key) exprTexts.push(n.key.text);
      if (n.bind) exprTexts.push(n.bind.text);
    } else if (n.kind === 'block-tag') {
      if (n.expr) exprTexts.push(n.expr.text);
    }
  }
  const setupMembers = new Set((setup?.members ?? []).map(m => m.name));
  const namespaceAliases = new Set((setup?.namespaces ?? []).map(ns => ns.alias));
  const blockLocals = collectBlockLocals(nodes);
  const signals = new Set<string>();
  const namespaces = new Set<string>();
  const unknown = new Set<string>();
  for (const txt of exprTexts) {
    for (const ident of extractIdents(txt)) {
      if (SKIP_IDENTS.has(ident)) continue;
      if (blockLocals.has(ident)) continue;
      if (namespaceAliases.has(ident)) { namespaces.add(ident); continue; }
      if (setupMembers.has(ident)) { signals.add(ident); continue; }
      unknown.add(ident);
    }
  }
  return {
    file,
    tags: [...tags].sort(),
    signals: [...signals].sort(),
    scope_unknown: [...unknown].sort(),
    events: [...events].sort(),
    bindings: [...bindings].sort(),
    namespaces: [...namespaces].sort(),
    controller: setup?.uri && setup.uri !== file ? setup.uri : null,
  };
}

function collectBlockLocals(nodes: Node[]): Set<string> {
  const locals = new Set<string>();
  for (const n of nodes) {
    if (n.kind === 'block-open' && n.block === 'for' && n.pattern) {
      for (const id of n.pattern.idents) locals.add(id);
    }
    if (n.kind === 'block-open' && n.block === 'while-error' && n.errorVar) {
      locals.add(n.errorVar.text);
    }
    if (n.kind === 'element-open') {
      for (const a of n.attrs) {
        if (a.kind === 'let') locals.add(a.name);
      }
    }
  }
  return locals;
}

function extractIdents(expr: string): string[] {
  // Strip string literals so `'$selected'` doesn't leak `selected`.
  const stripped = expr
    .replace(/'(?:[^'\\]|\\.)*'/g, "''")
    .replace(/"(?:[^"\\]|\\.)*"/g, '""')
    .replace(/`(?:[^`\\]|\\.)*`/g, '``');
  const out: string[] = [];
  let m: RegExpExecArray | null;
  IDENT_RE.lastIndex = 0;
  while ((m = IDENT_RE.exec(stripped)) !== null) {
    if (m.index > 0 && stripped[m.index - 1] === '.') continue;
    out.push(m[0]);
  }
  return out;
}

function main(): void {
  let parsed: ParsedArgs;
  try {
    parsed = parseArgs(process.argv.slice(2));
  } catch (e) {
    process.stderr.write(JSON.stringify({ error: (e as Error).message }) + '\n');
    process.exit(1);
    return;
  }
  switch (parsed.subcommand) {
    case 'diagnostics':
      runDiagnostics(resolveFile(parsed.file));
      break;
    case 'hover':
      if (parsed.line == null || parsed.col == null) {
        process.stderr.write(JSON.stringify({ error: 'hover requires --line and --col' }) + '\n');
        process.exit(1);
        return;
      }
      runHover(resolveFile(parsed.file), parsed.line, parsed.col);
      break;
    case 'dependencies':
      runDependencies(resolveFile(parsed.file));
      break;
    default:
      process.stderr.write(JSON.stringify({ error: `unknown subcommand: ${parsed.subcommand}` }) + '\n');
      process.exit(1);
  }
}

// Locate the paired controller for a `.sui` template. Tries, in order:
//   1. `<stem>.ts`              — the `loadSetup` default
//   2. `<stem>.controller.ts`   — sui-* component convention
//   3. `<dir>/controller.ts`    — views/<view>/page.sui convention
// Returns the first that exists, analyzed; null if none.
function resolveSetup(file: string): SetupInfo | null {
  const direct = loadSetup(file);
  if (direct && hasMembers(direct)) return direct;
  if (!file.endsWith('.sui')) return direct;
  const stem = file.slice(0, -'.sui'.length);
  for (const candidate of [`${stem}.controller.ts`, path.join(path.dirname(file), 'controller.ts')]) {
    if (!fs.existsSync(candidate)) continue;
    let source: string;
    try { source = fs.readFileSync(candidate, 'utf8'); } catch { continue; }
    const info = analyzeSetup(candidate, source);
    if (hasMembers(info)) return info;
  }
  return direct;
}

function hasMembers(s: SetupInfo): boolean {
  if (s.members.length > 0) return true;
  if ((s.namespaces?.length ?? 0) > 0) return true;
  if ((s.onHandlers?.length ?? 0) > 0) return true;
  return false;
}

function resolveFile(file: string): string {
  if (file.startsWith('file://')) {
    try { return url.fileURLToPath(file); } catch { /* fall through */ }
  }
  return file;
}

if (require.main === module) {
  main();
}

export { parseArgs, runDiagnostics, runHover, runDependencies, collectDependencies };
