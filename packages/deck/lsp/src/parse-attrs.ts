import type { Attribute, AttrKind, ParseError, Range } from './parse-types';
import { readBraceContent, readString } from './parse-braces';

const ATTR_NAME = /^[a-zA-Z_][a-zA-Z0-9_:-]*/;
const ATTR_LOCAL = /^[a-zA-Z_][a-zA-Z0-9_-]*/;

const PREFIXES: Record<string, AttrKind> = {
  'bind:': 'bind',
  'on:': 'on',
  'class:': 'class',
  'style:': 'style',
  'attr:': 'attr',
  'let:': 'let',
};

export function readAttribute(
  s: string, i: number, attrs: Attribute[], errors: ParseError[],
): number {
  for (const [pfx, k] of Object.entries(PREFIXES)) {
    if (s.startsWith(pfx, i)) return readPrefixedAttr(s, i, pfx, k, attrs, errors);
  }
  return readPlainAttr(s, i, attrs, errors);
}

function readPrefixedAttr(
  s: string, i: number, pfx: string, k: AttrKind,
  attrs: Attribute[], errors: ParseError[],
): number {
  let q = i + pfx.length;
  if (k === 'attr' && s[q] === '{') return readAttrAttr(s, i, q, attrs, errors);
  const lm = s.slice(q).match(ATTR_LOCAL);
  if (!lm) {
    errors.push({ range: { start: i, end: q }, message: `Expected name after \`${pfx}\`` });
    return q;
  }
  const name = lm[0];
  const nameRange: Range = { start: q, end: q + name.length };
  q += name.length;
  const modifiers: string[] = [];
  const modifierRanges: Range[] = [];
  if (k === 'on') q = readEventModifiers(s, q, modifiers, modifierRanges);
  let value: Attribute['value'];
  let outerValueRange: Range | undefined;
  if (k !== 'let' && s[q] === '=') {
    q++;
    if (s[q] === '{') {
      const v = readBraceContent(s, q, errors);
      value = {
        kind: 'expr',
        text: s.slice(v.start + 1, v.end - 1),
        range: { start: v.start + 1, end: v.end - 1 },
      };
      outerValueRange = v;
      q = v.end;
    } else if (s[q] === '"' || s[q] === "'") {
      const sv = readString(s, q, errors);
      value = {
        kind: 'string',
        text: s.slice(sv.start + 1, sv.end - 1),
        range: { start: sv.start + 1, end: sv.end - 1 },
      };
      outerValueRange = sv;
      q = sv.end;
    } else {
      errors.push({ range: { start: q, end: q + 1 }, message: 'Expected `{` or quoted string after `=`' });
    }
  }
  attrs.push({ kind: k, name, nameRange, modifiers, modifierRanges, value, outerValueRange, range: { start: i, end: q } });
  return q;
}

function readEventModifiers(
  s: string, qIn: number, modifiers: string[], modifierRanges: Range[],
): number {
  let q = qIn;
  while (s[q] === '|') {
    q++;
    const mm = s.slice(q).match(ATTR_LOCAL);
    if (!mm) break;
    modifiers.push(mm[0]);
    modifierRanges.push({ start: q, end: q + mm[0].length });
    q += mm[0].length;
    if (s[q] === ':') {
      q++;
      while (q < s.length && /[0-9]/.test(s[q]!)) q++;
    }
  }
  return q;
}

function readAttrAttr(
  s: string, i: number, qIn: number,
  attrs: Attribute[], errors: ParseError[],
): number {
  const exprRange = readBraceContent(s, qIn, errors);
  const nameExpr = {
    text: s.slice(exprRange.start + 1, exprRange.end - 1),
    range: exprRange,
  };
  let q = exprRange.end;
  let value: Attribute['value'];
  let outerValueRange: Range | undefined;
  if (s[q] === '=') {
    q++;
    if (s[q] === '{') {
      const v = readBraceContent(s, q, errors);
      value = {
        kind: 'expr',
        text: s.slice(v.start + 1, v.end - 1),
        range: { start: v.start + 1, end: v.end - 1 },
      };
      outerValueRange = v;
      q = v.end;
    }
  }
  attrs.push({
    kind: 'attr',
    name: '',
    nameRange: { start: i + 'attr:'.length, end: q },
    modifiers: [],
    modifierRanges: [],
    value,
    outerValueRange,
    nameExpr,
    range: { start: i, end: q },
  });
  return q;
}

function readPlainAttr(
  s: string, i: number, attrs: Attribute[], errors: ParseError[],
): number {
  const m = s.slice(i).match(ATTR_NAME);
  if (!m) return i;
  const name = m[0];
  const nameRange: Range = { start: i, end: i + name.length };
  let q = i + name.length;
  if (s[q] !== '=') {
    attrs.push({ kind: 'boolean', name, nameRange, modifiers: [], modifierRanges: [], range: { start: i, end: q } });
    return q;
  }
  q++;
  if (s[q] === '{') {
    const v = readBraceContent(s, q, errors);
    attrs.push({
      kind: 'dynamic', name, nameRange, modifiers: [], modifierRanges: [],
      value: { kind: 'expr', text: s.slice(v.start + 1, v.end - 1), range: { start: v.start + 1, end: v.end - 1 } },
      outerValueRange: v, range: { start: i, end: v.end },
    });
    return v.end;
  }
  if (s[q] === '"' || s[q] === "'") {
    const sv = readString(s, q, errors);
    attrs.push({
      kind: 'static', name, nameRange, modifiers: [], modifierRanges: [],
      value: { kind: 'string', text: s.slice(sv.start + 1, sv.end - 1), range: { start: sv.start + 1, end: sv.end - 1 } },
      outerValueRange: sv, range: { start: i, end: sv.end },
    });
    return sv.end;
  }
  errors.push({ range: { start: q, end: q + 1 }, message: 'Expected `{` or quoted string after `=`' });
  return q;
}
