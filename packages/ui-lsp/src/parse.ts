// Hand-rolled scanner for `.sui` templates.
//
// The shape mirrors the tree-sitter-set grammar (see
// `grammars/tree-sitter-set/grammar.js`) but is implemented in pure
// TypeScript so the LSP can run with zero native deps.

import type { Attribute, Node, ParseError, ParseResult, Range } from './parse-types';
import { readAttribute } from './parse-attrs';
import { readBraceConstruct } from './parse-blocks';

export * from './parse-types';

const WS = /\s/;
const TAG_NAME = /^[a-zA-Z][a-zA-Z0-9-]*/;

export function parse(source: string): ParseResult {
  const nodes: Node[] = [];
  const errors: ParseError[] = [];
  let i = 0;
  while (i < source.length) {
    const ch = source[i];
    if (ch === '<') {
      if (source.startsWith('<!--', i)) { i = readComment(source, i, nodes, errors); continue; }
      if (source[i + 1] === '/') { i = readCloseTag(source, i, nodes, errors); continue; }
      if (source[i + 1] && /[a-zA-Z]/.test(source[i + 1]!)) {
        i = readOpenTag(source, i, nodes, errors); continue;
      }
      i = readText(source, i, nodes); continue;
    }
    if (ch === '{') { i = readBraceConstruct(source, i, nodes, errors); continue; }
    i = readText(source, i, nodes);
  }
  return { source, nodes, errors };
}

function readComment(s: string, i: number, nodes: Node[], errors: ParseError[]): number {
  const start = i;
  const end = s.indexOf('-->', i + 4);
  if (end === -1) {
    errors.push({ range: { start, end: s.length }, message: 'Unterminated comment' });
    nodes.push({ kind: 'comment', range: { start, end: s.length } });
    return s.length;
  }
  const finish = end + 3;
  nodes.push({ kind: 'comment', range: { start, end: finish } });
  return finish;
}

function readCloseTag(s: string, i: number, nodes: Node[], errors: ParseError[]): number {
  const start = i;
  let p = i + 2;
  const m = s.slice(p).match(TAG_NAME);
  if (!m) {
    errors.push({ range: { start, end: p }, message: 'Expected tag name after `</`' });
    return p;
  }
  const tag = m[0];
  p += tag.length;
  while (p < s.length && s[p] !== '>') p++;
  if (s[p] !== '>') {
    errors.push({ range: { start, end: p }, message: 'Unterminated close tag' });
    return p;
  }
  p++;
  nodes.push({ kind: 'element-close', tag, range: { start, end: p } });
  return p;
}

function readOpenTag(s: string, i: number, nodes: Node[], errors: ParseError[]): number {
  const start = i;
  let p = i + 1;
  const m = s.slice(p).match(TAG_NAME);
  if (!m) {
    errors.push({ range: { start, end: p }, message: 'Expected tag name' });
    return p + 1;
  }
  const tag = m[0];
  const tagRange: Range = { start: p, end: p + tag.length };
  p += tag.length;
  const attrs: Attribute[] = [];
  while (p < s.length) {
    while (p < s.length && WS.test(s[p]!)) p++;
    if (p >= s.length) break;
    if (s[p] === '/' && s[p + 1] === '>') {
      p += 2;
      const range = { start, end: p };
      nodes.push({ kind: 'element-open', tag, tagRange, attrs, selfClosing: true, range });
      return p;
    }
    if (s[p] === '>') {
      p++;
      const range = { start, end: p };
      nodes.push({ kind: 'element-open', tag, tagRange, attrs, selfClosing: false, range });
      return p;
    }
    const before = p;
    p = readAttribute(s, p, attrs, errors);
    if (p === before) p++;
  }
  errors.push({ range: { start, end: p }, message: 'Unterminated open tag' });
  return p;
}

function readText(s: string, i: number, nodes: Node[]): number {
  const start = i;
  let p = i;
  while (p < s.length && s[p] !== '<' && s[p] !== '{') p++;
  if (p === start) return start + 1;
  nodes.push({ kind: 'text', range: { start, end: p } });
  return p;
}
