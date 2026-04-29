import type { Node, ParseError, Range } from './parse-types';
import { readBraceContent, scanExprUntilCloseBrace, scanIterUntilKeyOrClose } from './parse-braces';

const WS = /\s/;

export function readBraceConstruct(
  s: string, i: number, nodes: Node[], errors: ParseError[],
): number {
  if (s.startsWith('{#if', i)) return readSimpleBlockOpen(s, i, 'if', '{#if', nodes, errors);
  if (s.startsWith('{#match', i)) return readSimpleBlockOpen(s, i, 'match', '{#match', nodes, errors);
  if (s.startsWith('{#for', i)) return readForBlock(s, i, nodes, errors);
  if (s[i + 1] === ':' || s[i + 1] === '/') return readBlockTagOrClose(s, i, nodes, errors);
  const r = readBraceContent(s, i, errors);
  nodes.push({
    kind: 'interpolation',
    text: s.slice(r.start + 1, r.end - 1),
    range: { start: r.start + 1, end: r.end - 1 },
    outerRange: r,
  });
  return r.end;
}

function readSimpleBlockOpen(
  s: string, i: number, kind: 'if' | 'match', head: string,
  nodes: Node[], errors: ParseError[],
): number {
  const start = i;
  let p = i + head.length;
  const r = scanExprUntilCloseBrace(s, p, errors);
  const expr = { text: s.slice(p, r.end).trim(), range: { start: p, end: r.end } };
  let q = r.end;
  if (s[q] === '}') q++;
  nodes.push({ kind: 'block-open', block: kind, expr, outerRange: { start, end: q } });
  return q;
}

function readForBlock(s: string, i: number, nodes: Node[], errors: ParseError[]): number {
  const start = i;
  let p = i + '{#for'.length;
  while (p < s.length && WS.test(s[p]!)) p++;
  const idents: string[] = [];
  const patStart = p;
  const idRe = /^[a-zA-Z_$][a-zA-Z0-9_$]*/;
  while (true) {
    const m = s.slice(p).match(idRe);
    if (!m) break;
    idents.push(m[0]);
    p += m[0].length;
    while (p < s.length && WS.test(s[p]!)) p++;
    if (s[p] === ',') { p++; while (p < s.length && WS.test(s[p]!)) p++; continue; }
    break;
  }
  const patternRange = { start: patStart, end: p };
  if (s.startsWith('of', p)) p += 2;
  while (p < s.length && WS.test(s[p]!)) p++;
  const iterStart = p;
  const { end: iterEnd, sawKey } = scanIterUntilKeyOrClose(s, p, errors);
  const iter = { text: s.slice(iterStart, iterEnd).trim(), range: { start: iterStart, end: iterEnd } };
  p = iterEnd;
  let key: { text: string; range: Range } | undefined;
  if (sawKey) {
    while (p < s.length && WS.test(s[p]!)) p++;
    p += 'key'.length;
    while (p < s.length && WS.test(s[p]!)) p++;
    if (s[p] === '=') p++;
    if (s[p] === '{') {
      const k = readBraceContent(s, p, errors);
      key = { text: s.slice(k.start + 1, k.end - 1), range: k };
      p = k.end;
    }
  }
  while (p < s.length && WS.test(s[p]!)) p++;
  if (s[p] === '}') p++;
  nodes.push({
    kind: 'block-open', block: 'for',
    pattern: { idents, range: patternRange },
    iter, key,
    outerRange: { start, end: p },
  });
  return p;
}

function readBlockTagOrClose(s: string, i: number, nodes: Node[], errors: ParseError[]): number {
  if (s.startsWith('{:else if', i)) return readClause(s, i, '{:else if', 'elif', nodes, errors);
  if (s.startsWith('{:elif', i)) return readClause(s, i, '{:elif', 'elif', nodes, errors);
  if (s.startsWith('{:case', i)) return readClause(s, i, '{:case', 'case', nodes, errors);
  if (s.startsWith('{:else}', i)) {
    const end = i + '{:else}'.length;
    nodes.push({ kind: 'block-tag', tag: 'else', outerRange: { start: i, end } });
    return end;
  }
  if (s.startsWith('{:default}', i)) {
    const end = i + '{:default}'.length;
    nodes.push({ kind: 'block-tag', tag: 'default', outerRange: { start: i, end } });
    return end;
  }
  for (const [tok, b] of [['{/if}', 'if'], ['{/for}', 'for'], ['{/match}', 'match']] as const) {
    if (s.startsWith(tok, i)) {
      const end = i + tok.length;
      nodes.push({ kind: 'block-close', block: b, outerRange: { start: i, end } });
      return end;
    }
  }
  errors.push({ range: { start: i, end: i + 1 }, message: 'Unrecognised brace construct' });
  return i + 1;
}

function readClause(
  s: string, i: number, head: string,
  tag: 'elif' | 'case', nodes: Node[], errors: ParseError[],
): number {
  const start = i;
  let p = i + head.length;
  const r = scanExprUntilCloseBrace(s, p, errors);
  const expr = { text: s.slice(p, r.end).trim(), range: { start: p, end: r.end } };
  let q = r.end;
  if (s[q] === '}') q++;
  nodes.push({ kind: 'block-tag', tag, expr, outerRange: { start, end: q } });
  return q;
}
