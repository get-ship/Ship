import type { ParseError, Range } from './parse-types';

export function readBraceContent(s: string, i: number, errors: ParseError[]): Range {
  if (s[i] !== '{') return { start: i, end: i };
  const start = i;
  let p = i + 1;
  let depth = 1;
  while (p < s.length && depth > 0) {
    const c = s[p]!;
    if (c === '"' || c === "'" || c === '`') { p = skipStringLit(s, p); continue; }
    if (c === '{' || c === '(' || c === '[') depth++;
    else if (c === '}' || c === ')' || c === ']') depth--;
    if (depth === 0) { p++; break; }
    p++;
  }
  if (depth > 0) errors.push({ range: { start, end: s.length }, message: 'Unterminated `{` expression' });
  return { start, end: p };
}

export function skipStringLit(s: string, i: number): number {
  const quote = s[i]!;
  let p = i + 1;
  while (p < s.length) {
    const c = s[p]!;
    if (c === '\\') { p += 2; continue; }
    if (c === quote) return p + 1;
    p++;
  }
  return s.length;
}

export function readString(s: string, i: number, errors: ParseError[]): Range {
  const quote = s[i]!;
  const start = i;
  let p = i + 1;
  while (p < s.length && s[p] !== quote) p++;
  if (s[p] !== quote) errors.push({ range: { start, end: p }, message: 'Unterminated string' });
  return { start, end: p + 1 };
}

export function scanExprUntilCloseBrace(s: string, i: number, errors: ParseError[]): { end: number } {
  let p = i;
  let depth = 0;
  while (p < s.length) {
    const c = s[p]!;
    if (c === '"' || c === "'" || c === '`') { p = skipStringLit(s, p); continue; }
    if (c === '{' || c === '(' || c === '[') { depth++; p++; continue; }
    if (c === ')' || c === ']') { depth--; p++; continue; }
    if (c === '}') {
      if (depth === 0) return { end: p };
      depth--; p++; continue;
    }
    p++;
  }
  errors.push({ range: { start: i, end: s.length }, message: 'Unterminated brace block' });
  return { end: s.length };
}

export function scanIterUntilKeyOrClose(
  s: string, i: number, errors: ParseError[],
): { end: number; sawKey: boolean } {
  let p = i;
  let depth = 0;
  while (p < s.length) {
    const c = s[p]!;
    if (c === '"' || c === "'" || c === '`') { p = skipStringLit(s, p); continue; }
    if (c === '{' || c === '(' || c === '[') { depth++; p++; continue; }
    if (c === ')' || c === ']') { depth--; p++; continue; }
    if (c === '}') {
      if (depth === 0) return { end: p, sawKey: false };
      depth--; p++; continue;
    }
    if (depth === 0 && /\s/.test(c) && s.startsWith('key', p + 1) && /[=\s]/.test(s[p + 4] ?? '')) {
      return { end: p, sawKey: true };
    }
    p++;
  }
  errors.push({ range: { start: i, end: s.length }, message: 'Unterminated for-block iterator' });
  return { end: s.length, sawKey: false };
}
