// Tokenize a parent's child nodes into a flat token stream where text
// is split at template-grammar markers. Block markers (`{#if}`,
// `{:else}`, `{/if}`, etc.) become discrete tokens; element children
// pass through untouched as `element` tokens. The result is consumed
// by template/compile.ts which builds the instruction tree.

/** One unit of compiled-template input. */
export type Token =
  | { kind: 'text'; value: string }
  | { kind: 'expr'; src: string; loc: string }
  | { kind: 'element'; node: Element }
  | { kind: 'comment'; node: Comment }
  | { kind: 'open-if'; src: string; loc: string }
  | { kind: 'open-for'; src: string; loc: string }
  | { kind: 'open-match'; src: string; loc: string }
  | { kind: 'else-if'; src: string; loc: string }
  | { kind: 'else'; loc: string }
  | { kind: 'case'; src: string; loc: string }
  | { kind: 'default'; loc: string }
  | { kind: 'close-if'; loc: string }
  | { kind: 'close-for'; loc: string }
  | { kind: 'close-match'; loc: string }

/** Walk `parent`'s children and produce the flat token stream. */
export function tokenizeChildren(parent: Node): Token[] {
  const out: Token[] = []
  for (const node of Array.from(parent.childNodes)) {
    if (node.nodeType === 3 /* TEXT_NODE */) {
      tokenizeText((node as Text).data, out)
    } else if (node.nodeType === 8 /* COMMENT_NODE */) {
      out.push({ kind: 'comment', node: node as Comment })
    } else if (node.nodeType === 1 /* ELEMENT_NODE */) {
      out.push({ kind: 'element', node: node as Element })
    }
  }
  return out
}

/**
 * Tokenize a single text-node's content, pushing tokens onto `out`.
 * Recognizes `{...}` expressions and the block-marker forms.
 */
export function tokenizeText(text: string, out: Token[]): void {
  let i = 0
  let buf = ''
  const len = text.length
  while (i < len) {
    if (text[i] === '{') {
      const close = findMatchingClose(text, i)
      if (close === -1) {
        // Unmatched — treat literally; don't crash on stray `{`.
        buf += text[i]
        i++
        continue
      }
      const inner = text.slice(i + 1, close).trim()
      // Push any buffered plain text first.
      if (buf.length > 0) {
        out.push({ kind: 'text', value: buf })
        buf = ''
      }
      out.push(classifyBrace(inner, text.slice(i, close + 1)))
      i = close + 1
      continue
    }
    buf += text[i]
    i++
  }
  if (buf.length > 0) out.push({ kind: 'text', value: buf })
}

/** Locate the matching `}` for a `{` at position `start`. */
function findMatchingClose(text: string, start: number): number {
  let depth = 0
  let str: '"' | "'" | '`' | null = null
  for (let i = start; i < text.length; i++) {
    const c = text[i]!
    if (str) {
      if (c === '\\') {
        i++
        continue
      }
      if (c === str) str = null
      continue
    }
    if (c === '"' || c === "'" || c === '`') {
      str = c
      continue
    }
    if (c === '{') depth++
    else if (c === '}') {
      depth--
      if (depth === 0) return i
    }
  }
  return -1
}

/** Turn the text inside `{...}` into the appropriate token. */
function classifyBrace(inner: string, raw: string): Token {
  const loc = raw.length > 60 ? raw.slice(0, 57) + '…' : raw

  if (inner.startsWith('#if')) {
    return { kind: 'open-if', src: inner.slice(3).trim(), loc }
  }
  if (inner.startsWith('#for')) {
    return { kind: 'open-for', src: inner.slice(4).trim(), loc }
  }
  if (inner.startsWith('#match')) {
    return { kind: 'open-match', src: inner.slice(6).trim(), loc }
  }
  if (inner.startsWith(':else if')) {
    return { kind: 'else-if', src: inner.slice(8).trim(), loc }
  }
  if (inner === ':else') {
    return { kind: 'else', loc }
  }
  if (inner.startsWith(':case')) {
    return { kind: 'case', src: inner.slice(5).trim(), loc }
  }
  if (inner === ':default') {
    return { kind: 'default', loc }
  }
  if (inner === '/if') return { kind: 'close-if', loc }
  if (inner === '/for') return { kind: 'close-for', loc }
  if (inner === '/match') return { kind: 'close-match', loc }
  return { kind: 'expr', src: inner, loc }
}
