// Position-driven completion for `.sui`.
//
// Looks at the source slice up to the cursor and decides which menu
// to show:
//   • tag completion          — after `<` (typing element name)
//   • directive completion    — after `<d` etc., specifically suggests
//                               `<data />`, `<state />`, `<while-…>`, …
//   • attribute prefix locals — after `bind:`, `on:`, `class:`, `style:`,
//                               `attr:`, `let:`
//   • event modifier          — after `on:click|`
//   • identifier completion   — when cursor sits inside a `{…}`
//                               expression / block body

import * as fs from 'node:fs';
import * as path from 'node:path';
import type { Node } from './parse-types';
import { buildScope, VIEW_GLOBALS } from './bindings';
import type { SetupInfo } from './setup';

export type CompletionKind = 'tag' | 'directive' | 'attr-local' | 'event-modifier' | 'identifier' | 'attr-name';

export interface CompletionItem {
  label: string;
  kind: CompletionKind;
  detail?: string;
  insertText?: string;
}

export interface CompletionContext {
  source: string;
  offset: number;
  nodes: Node[];
  setup: SetupInfo | null;
  primitiveTags?: string[];
}

const DIRECTIVE_TAGS = ['data', 'state', 'component', 'prop', 'while-loading', 'while-error', 'slot'];
const HTML_TAGS = ['div', 'span', 'p', 'a', 'button', 'input', 'form', 'ul', 'li', 'ol', 'header', 'footer', 'main', 'section', 'article', 'h1', 'h2', 'h3', 'h4', 'img', 'br', 'hr', 'code', 'pre'];

const BIND_LOCALS = ['value', 'checked', 'group', 'open', 'this', 'files'];
const EVENT_NAMES = [
  'click', 'input', 'change', 'submit', 'focus', 'blur', 'keydown', 'keyup',
  'keypress', 'mousedown', 'mouseup', 'mouseover', 'mouseout', 'mouseenter',
  'mouseleave', 'pointerdown', 'pointerup', 'pointermove', 'wheel',
  'dragstart', 'dragend', 'drop', 'load', 'error',
];
const EVENT_MODIFIERS = ['preventDefault', 'stop', 'stopImmediate', 'capture', 'self', 'once', 'passive', 'debounce', 'throttle'];
const STYLE_LOCALS = ['color', 'background', 'background-color', 'font-size', 'padding', 'margin', 'width', 'height', 'display', 'opacity', 'transform', '--tone'];

export function complete(ctx: CompletionContext): CompletionItem[] {
  const { source, offset } = ctx;
  const before = source.slice(0, offset);
  // Inside a brace expression?
  const braceCtx = innermostOpenBrace(before);
  if (braceCtx) {
    // `<alias>.` member-access completion (D-2).
    const dotted = /([a-zA-Z_$][a-zA-Z0-9_$]*)\.([a-zA-Z_$][a-zA-Z0-9_$]*)?$/.exec(before);
    if (dotted) {
      const ns = ctx.setup?.namespaces?.find(n => n.alias === dotted[1]);
      if (ns) return namespaceMemberCompletions(ns);
    }
    // Inside an `on:event={…}` brace — restrict to controller `on` keys (D-4).
    const onCtx = onAttrBraceContext(before);
    if (onCtx) return onHandlerCompletions(ctx);
    return identifierCompletions(ctx);
  }

  // Inside an open tag (cursor between `<tag` and `>`)?
  const openTag = openTagBeforeCursor(before);
  if (openTag) return attrAreaCompletions(ctx, before, openTag);

  // Tag-name position
  const tagMatch = /<([a-zA-Z][a-zA-Z0-9-]*)?$/.exec(before);
  if (tagMatch) return tagCompletions(ctx);

  return [];
}

/** True if the cursor sits inside the `{…}` of an `on:event={…}` attr. */
function onAttrBraceContext(before: string): boolean {
  // Walk back to the opening `{` of the current brace expression.
  let depth = 0;
  let p = before.length - 1;
  while (p >= 0) {
    const c = before[p]!;
    if (c === '"' || c === "'" || c === '`') { p = skipBackString(before, p); continue; }
    if (c === '}' || c === ')' || c === ']') depth++;
    else if (c === '(' || c === '[') depth--;
    else if (c === '{') {
      if (depth === 0) {
        // The `{` is the brace start. Look at the chars right before it.
        const pre = before.slice(0, p).trimEnd();
        return /on:[a-zA-Z][a-zA-Z0-9-]*(?:\|[a-zA-Z][a-zA-Z0-9-]*(?::[0-9]+)?)*=$/.test(pre);
      }
      depth--;
    }
    p--;
  }
  return false;
}

function namespaceMemberCompletions(ns: import('./setup').UseNamespace): CompletionItem[] {
  return ns.members.map(m => ({
    label: m.name,
    kind: 'identifier' as const,
    detail: `${m.kind}: ${m.signature}`,
  }));
}

function onHandlerCompletions(ctx: CompletionContext): CompletionItem[] {
  const handlers = ctx.setup?.onHandlers ?? [];
  return handlers.map(h => ({
    label: h.name,
    kind: 'identifier' as const,
    detail: h.signature,
  }));
}

function innermostOpenBrace(src: string): boolean {
  // Walk backward to determine whether an opening `{` (interpolation
  // or `{#…}`) is currently unclosed at the cursor. Honors string
  // literals and balanced `()` `[]` `{}`.
  let depthCurly = 0;
  let p = src.length - 1;
  while (p >= 0) {
    const c = src[p]!;
    if (c === '"' || c === "'" || c === '`') { p = skipBackString(src, p); continue; }
    if (c === '}' || c === ')' || c === ']') depthCurly++;
    else if (c === '{') {
      if (depthCurly === 0) return true;
      depthCurly--;
      // Stop scanning past a tag boundary (don't peer past `<…>`)
      // — when we hit a `<` outside of any open brace, we know we're
      // not inside a brace expression.
    } else if (c === '(' || c === '[') {
      depthCurly--;
    } else if (c === '<' && depthCurly === 0) {
      return false;
    }
    p--;
  }
  return false;
}

function skipBackString(src: string, p: number): number {
  const quote = src[p]!;
  let q = p - 1;
  while (q >= 0) {
    if (src[q] === quote && src[q - 1] !== '\\') return q - 1;
    q--;
  }
  return -1;
}

function openTagBeforeCursor(src: string): { tag: string; tagStart: number } | null {
  // Walk backward to find an unclosed `<tag…` (no `>` after it). Only
  // considers the cursor to be in the attribute area if the tag name
  // ended (whitespace or another non-identifier char appears between
  // the tag name and the cursor).
  for (let p = src.length - 1; p >= 0; p--) {
    const c = src[p]!;
    if (c === '>') return null;
    if (c === '<') {
      const m = /^<([a-zA-Z][a-zA-Z0-9-]*)/.exec(src.slice(p));
      if (!m) return null;
      const tagEnd = p + 1 + m[1]!.length;
      if (tagEnd >= src.length) return null;
      const after = src.slice(tagEnd);
      if (!/^\s/.test(after)) return null;
      return { tag: m[1]!, tagStart: p };
    }
  }
  return null;
}

function tagCompletions(_ctx: CompletionContext): CompletionItem[] {
  const out: CompletionItem[] = [];
  for (const t of DIRECTIVE_TAGS) {
    out.push({ label: t, kind: 'directive', detail: `directive element <${t}>`, insertText: t });
  }
  const prims = _ctx.primitiveTags ?? defaultPrimitiveTags();
  for (const p of prims) out.push({ label: p, kind: 'tag', detail: 'ship-set primitive' });
  for (const h of HTML_TAGS) out.push({ label: h, kind: 'tag', detail: 'html element' });
  return out;
}

function attrAreaCompletions(
  ctx: CompletionContext, before: string, _openTag: { tag: string; tagStart: number },
): CompletionItem[] {
  // After `bind:`, `on:`, `class:`, `style:`, `attr:`, `let:`?
  const m = /(bind|on|class|style|attr|let):([a-zA-Z_-]*)$/.exec(before);
  if (m) return attrLocalCompletions(m[1] as 'bind' | 'on' | 'class' | 'style' | 'attr' | 'let');
  // Event modifier? `on:click|...`
  const mm = /on:[a-zA-Z]+(?:\|[a-zA-Z]+)*\|([a-zA-Z]*)$/.exec(before);
  if (mm) return EVENT_MODIFIERS.map(name => ({ label: name, kind: 'event-modifier' as const }));
  // Otherwise: attribute name suggestions including the prefixes.
  const out: CompletionItem[] = [];
  for (const k of ['bind:', 'on:', 'class:', 'style:', 'attr:', 'let:']) {
    out.push({ label: k, kind: 'attr-name' });
  }
  for (const a of ['id', 'class', 'style', 'name', 'data-test', 'aria-label']) {
    out.push({ label: a, kind: 'attr-name' });
  }
  // Suppress identifier list here — only fires inside `{…}`.
  void ctx;
  return out;
}

function attrLocalCompletions(prefix: 'bind' | 'on' | 'class' | 'style' | 'attr' | 'let'): CompletionItem[] {
  if (prefix === 'bind') return BIND_LOCALS.map(n => ({ label: n, kind: 'attr-local' }));
  if (prefix === 'on') return EVENT_NAMES.map(n => ({ label: n, kind: 'attr-local' }));
  if (prefix === 'style') return STYLE_LOCALS.map(n => ({ label: n, kind: 'attr-local' }));
  if (prefix === 'class') return [];
  if (prefix === 'attr') return [];
  if (prefix === 'let') return [];
  return [];
}

function identifierCompletions(ctx: CompletionContext): CompletionItem[] {
  const scope = buildScope(ctx.nodes, ctx.source);
  const seen = new Set<string>();
  const out: CompletionItem[] = [];
  for (const d of scope.declarations) {
    if (ctx.offset < d.extent.start || ctx.offset > d.extent.end) continue;
    if (seen.has(d.name)) continue;
    seen.add(d.name);
    out.push({ label: d.name, kind: 'identifier', detail: d.source });
  }
  if (ctx.setup) {
    for (const m of ctx.setup.members) {
      if (seen.has(m.name)) continue;
      seen.add(m.name);
      out.push({ label: m.name, kind: 'identifier', detail: m.type });
    }
  }
  for (const g of VIEW_GLOBALS) {
    if (seen.has(g)) continue;
    seen.add(g);
    out.push({ label: g, kind: 'identifier', detail: 'global' });
  }
  return out;
}

let primitiveCache: string[] | null = null;

export function defaultPrimitiveTags(): string[] {
  if (primitiveCache) return primitiveCache;
  const cemPath = findCemPath();
  if (!cemPath) return (primitiveCache = []);
  try {
    const raw = JSON.parse(fs.readFileSync(cemPath, 'utf8')) as { modules?: { declarations?: { kind?: string; tagName?: string }[] }[] };
    const tags = new Set<string>();
    for (const mod of raw.modules ?? []) {
      for (const d of mod.declarations ?? []) {
        if (d.kind === 'class' && d.tagName && d.tagName.startsWith('ship-')) tags.add(d.tagName);
      }
    }
    primitiveCache = [...tags].sort();
    return primitiveCache;
  } catch {
    return (primitiveCache = []);
  }
}

function findCemPath(): string | null {
  let dir = process.cwd();
  for (let i = 0; i < 6; i++) {
    const candidate = path.join(dir, 'packages', 'ship-set', 'custom-elements.json');
    if (fs.existsSync(candidate)) return candidate;
    const parent = path.dirname(dir);
    if (parent === dir) break;
    dir = parent;
  }
  return null;
}
