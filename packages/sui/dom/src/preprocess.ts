// Source preprocessor — papers over the few places where the
// `.sui` surface diverges from what an HTML5 parser accepts.
//
// Two transforms run before DOMParser sees the source:
//
//   1. Unquoted brace attributes — `disabled={x || y}` — get wrapped
//      so the spaces inside the expression don't terminate the
//      attribute value. The braces survive the wrap and are stripped
//      again by the attribute compiler.
//
//   2. Self-closing tags — `<state name="x" value="0" />` — only
//      auto-close in HTML for the void elements (br, hr, img, input,
//      …). Any other tag would otherwise wrap the rest of the file
//      as its descendants. We expand non-void self-closers to the
//      explicit `<tag …></tag>` form.

const VOID_TAGS = new Set([
  'area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input',
  'link', 'meta', 'source', 'track', 'wbr',
])

/** Run both transforms. */
export function preprocess(source: string): string {
  return expandSelfClose(quoteBraceAttrs(source))
}

/**
 * Convert ` attr={EXPR}` (no surrounding quotes) into
 * ` attr="{EXPR}"`, with any `"` inside the expression escaped to
 * `&quot;` so the surrounding quotes survive HTML attribute parsing.
 */
export function quoteBraceAttrs(source: string): string {
  let out = ''
  let i = 0
  const len = source.length
  while (i < len) {
    const c = source[i]!
    if (c !== '<') {
      out += c
      i++
      continue
    }
    // Scan until matching `>` while rewriting brace attrs inside.
    const tagStart = i
    out += '<'
    i++
    let inSingle = false
    let inDouble = false
    while (i < len) {
      const ch = source[i]!
      if (ch === '"' && !inSingle) {
        inDouble = !inDouble
        out += ch
        i++
        continue
      }
      if (ch === "'" && !inDouble) {
        inSingle = !inSingle
        out += ch
        i++
        continue
      }
      if (!inSingle && !inDouble && ch === '>') {
        out += '>'
        i++
        break
      }
      if (!inSingle && !inDouble && ch === '=' && source[i + 1] === '{') {
        const end = findClose(source, i + 1)
        if (end === -1) {
          out += ch
          i++
          continue
        }
        const expr = source.slice(i + 1, end + 1)
        out += '="' + expr.replace(/"/g, '&quot;') + '"'
        i = end + 1
        continue
      }
      out += ch
      i++
    }
    void tagStart
  }
  return out
}

const findClose = (s: string, openBraceAt: number): number => {
  let depth = 0
  let str: '"' | "'" | '`' | null = null
  for (let i = openBraceAt; i < s.length; i++) {
    const c = s[i]!
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

/**
 * Convert `<tag … />` into `<tag …></tag>` for non-void tags so the
 * HTML parser doesn't wrap subsequent siblings inside the unclosed
 * element.
 */
export function expandSelfClose(source: string): string {
  let out = ''
  let i = 0
  const len = source.length
  while (i < len) {
    const c = source[i]!
    if (c !== '<') {
      out += c
      i++
      continue
    }
    // Read a tag start, looking for `/>` ending. Skip `</…>` close
    // tags and `<!…>` directives entirely.
    if (source[i + 1] === '/' || source[i + 1] === '!') {
      const end = source.indexOf('>', i)
      if (end === -1) {
        out += source.slice(i)
        break
      }
      out += source.slice(i, end + 1)
      i = end + 1
      continue
    }
    const tagMatch = source.slice(i + 1).match(/^([A-Za-z][A-Za-z0-9-]*)/)
    if (!tagMatch) {
      out += c
      i++
      continue
    }
    const tag = tagMatch[1]!.toLowerCase()
    // Find matching `>` accounting for quoted attributes.
    let j = i + 1 + tagMatch[1]!.length
    let inSingle = false
    let inDouble = false
    while (j < len) {
      const ch = source[j]!
      if (ch === '"' && !inSingle) inDouble = !inDouble
      else if (ch === "'" && !inDouble) inSingle = !inSingle
      else if (!inSingle && !inDouble && ch === '>') break
      j++
    }
    if (j === len) {
      out += source.slice(i)
      break
    }
    const head = source.slice(i, j)
    const trimmedHead = head.replace(/\s*\/\s*$/, '')
    const isSelfClose = trimmedHead !== head
    if (isSelfClose && !VOID_TAGS.has(tag)) {
      out += trimmedHead + `></${tag}>`
    } else {
      out += source.slice(i, j + 1)
    }
    i = j + 1
  }
  return out
}
