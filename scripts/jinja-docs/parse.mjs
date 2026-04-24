/**
 * Parser for Ship's Jinja macro docstring convention.
 *
 * Source of truth: the `{#- ... -#}` (or `{# ... #}`) comment block
 * immediately preceding each `{% macro name(args...) %}` declaration.
 *
 * Structured shape parsed out of each comment:
 *
 *   - First non-blank line: `name(arg, arg2=default, ...)` — the signature line.
 *   - Following paragraph(s): short summary + extended description.
 *   - Optional `Args:` block: one entry per argument, indented, `name: description.`
 *   - Optional `Example:` block: verbatim copy-paste example.
 */

/**
 * Scan source text for doc-comment + macro pairs.
 *
 * We collect comment blocks and macro openers separately (each via a strict
 * non-nested regex), then pair each macro with the nearest preceding comment
 * that is separated from it by whitespace only.
 *
 * Returns { name, rawArgs, doc, source } per macro in source order.
 */
export function extractMacros(sourceText, sourcePath) {
  // Match individual comments. Use the fact that `#}` cannot appear inside a
  // comment: the capture group is non-`#}` characters until the terminator.
  const commentRe = /\{#(-?)([\s\S]*?)(-?)#\}/g;
  const comments = [];
  let cm;
  while ((cm = commentRe.exec(sourceText)) !== null) {
    comments.push({ start: cm.index, end: cm.index + cm[0].length, body: cm[2] });
  }

  // Locate each `{% macro name(...` and then scan forward to find the matching
  // close paren + `%}`. Simple char-by-char walk so default values containing
  // parens or strings don't confuse the parser.
  const macroOpenRe = /\{%\s*macro\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\(/g;
  const results = [];
  let m;
  while ((m = macroOpenRe.exec(sourceText)) !== null) {
    const name = m[1];
    const argStart = m.index + m[0].length;
    const argEnd = scanBalancedParen(sourceText, argStart);
    if (argEnd < 0) continue;
    const rawArgs = sourceText.slice(argStart, argEnd);
    const afterParen = sourceText.slice(argEnd + 1);
    const closeMatch = afterParen.match(/^\s*%\}/);
    if (!closeMatch) continue;
    const macroStartIndex = m.index;
    const line = sourceText.slice(0, macroStartIndex).split('\n').length;

    // Find the comment whose `end` immediately precedes this macro's `start`,
    // separated by whitespace only.
    let doc = null;
    for (let i = comments.length - 1; i >= 0; i--) {
      const c = comments[i];
      if (c.end > macroStartIndex) continue;
      const between = sourceText.slice(c.end, macroStartIndex);
      if (/^\s*$/.test(between)) {
        doc = stripCommentDecorations(c.body);
        break;
      }
      break; // nearest preceding comment isn't adjacent — no doc
    }

    // Reject comments that are section dividers or don't match this macro's
    // signature line — treat as undocumented.
    if (doc != null) {
      const first = firstNonBlankLine(doc);
      const looksLikeDivider = /^=+\s+.+\s+=+$/.test(first);
      const looksLikeHeader = new RegExp(`^${escapeRegExp(name)}\\s*\\(`).test(first);
      if (looksLikeDivider || !looksLikeHeader) {
        doc = null;
      }
    }

    results.push({
      name,
      rawArgs: rawArgs.trim(),
      doc,
      source: `${sourcePath}:${line}`,
    });
    // Ensure the outer regex resumes past the macro header so default values
    // containing `{% ... %}` fragments don't double-count.
    macroOpenRe.lastIndex = argEnd + closeMatch[0].length;
  }
  return results;
}

/**
 * Starting at `start` (one past an opening `(`), scan forward to the matching
 * close paren. Tracks string literals and nested brackets so default values
 * like `"'HTTP ' + (x || '?')"` don't trip the balance. Returns the index of
 * the closing `)` or -1 if unbalanced.
 */
function scanBalancedParen(text, start) {
  let depth = 1;
  let inSingle = false;
  let inDouble = false;
  for (let i = start; i < text.length; i++) {
    const ch = text[i];
    if (inSingle) {
      if (ch === "'" && text[i - 1] !== '\\') inSingle = false;
      continue;
    }
    if (inDouble) {
      if (ch === '"' && text[i - 1] !== '\\') inDouble = false;
      continue;
    }
    if (ch === "'") { inSingle = true; continue; }
    if (ch === '"') { inDouble = true; continue; }
    if (ch === '(') depth++;
    else if (ch === ')') {
      depth--;
      if (depth === 0) return i;
    }
  }
  return -1;
}

function stripCommentDecorations(doc) {
  // Comments use consistent 2-space indent on each line; strip it so the
  // parsed text is flush-left.
  const lines = doc.split('\n');
  const indent = minCommonIndent(lines.filter((l) => l.trim().length > 0));
  return lines.map((l) => l.slice(indent)).join('\n').trim();
}

function minCommonIndent(lines) {
  if (lines.length === 0) return 0;
  let min = Infinity;
  for (const l of lines) {
    const m = l.match(/^( *)/);
    const n = m ? m[1].length : 0;
    if (n < min) min = n;
  }
  return min === Infinity ? 0 : min;
}

function firstNonBlankLine(text) {
  for (const line of text.split('\n')) {
    const t = line.trim();
    if (t.length > 0) return t;
  }
  return '';
}

function escapeRegExp(s) {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/**
 * Turn a raw doc block into structured fields. Returns:
 *   { signature, summary, description, args: [{name, description}], example }
 * All fields may be null / empty when the doc is missing or sparse.
 */
export function parseDoc(doc) {
  if (doc == null) {
    return { signature: null, summary: null, description: null, args: [], example: null };
  }
  const lines = doc.split('\n');
  // Signature line = first non-blank.
  let signature = null;
  let idx = 0;
  while (idx < lines.length && lines[idx].trim() === '') idx++;
  if (idx < lines.length) {
    signature = lines[idx].trim();
    idx++;
  }

  // Walk remaining lines, splitting into: body paragraphs, Args block, Example block.
  const bodyLines = [];
  const argLines = [];
  const exampleLines = [];
  let mode = 'body';
  for (; idx < lines.length; idx++) {
    const line = lines[idx];
    const trimmed = line.trim();
    if (/^Args?:$/i.test(trimmed)) { mode = 'args'; continue; }
    if (/^Example:$/i.test(trimmed)) { mode = 'example'; continue; }
    if (mode === 'body') bodyLines.push(line);
    else if (mode === 'args') argLines.push(line);
    else exampleLines.push(line);
  }

  const description = bodyLines.join('\n').trim() || null;
  const summary = description ? description.split(/\n\s*\n/)[0].replace(/\s+/g, ' ').trim() : null;

  const args = parseArgsBlock(argLines);
  const example = dedent(exampleLines.join('\n')).replace(/^\n+|\n+$/g, '') || null;

  return { signature, summary, description, args, example };
}

function dedent(text) {
  const lines = text.split('\n');
  const nonEmpty = lines.filter((l) => l.trim().length > 0);
  if (nonEmpty.length === 0) return text;
  const indent = minCommonIndent(nonEmpty);
  return lines.map((l) => l.slice(indent)).join('\n');
}

function parseArgsBlock(lines) {
  // Each arg starts with `name:` at the leftmost indent in the block.
  // Continuation lines are indented further and get appended to the previous
  // arg's description.
  const nonEmpty = lines.filter((l) => l.trim().length > 0);
  if (nonEmpty.length === 0) return [];
  const baseIndent = minCommonIndent(nonEmpty);
  const entries = [];
  let current = null;
  for (const raw of lines) {
    if (raw.trim().length === 0) continue;
    const indent = raw.match(/^ */)[0].length;
    const content = raw.slice(baseIndent);
    const startMatch = content.match(/^([a-zA-Z_][a-zA-Z0-9_]*):\s*(.*)$/);
    if (indent === baseIndent && startMatch) {
      if (current) entries.push(current);
      current = { name: startMatch[1], description: startMatch[2].trim() };
    } else if (current) {
      current.description = (current.description + ' ' + content.trim()).trim();
    }
  }
  if (current) entries.push(current);
  return entries;
}

/**
 * Parse the macro parameter list from the Jinja header. Returns an array of
 * { name, required, default }. We don't try to infer types from Jinja —
 * callers doc the intent in the `Args:` block.
 */
export function parseParams(rawArgs) {
  if (!rawArgs.trim()) return [];
  // Split on commas NOT inside nested parens/quotes. The current macros keep
  // defaults simple enough that a simple state machine works.
  const params = [];
  let buf = '';
  let depth = 0;
  let inSingle = false;
  let inDouble = false;
  for (let i = 0; i < rawArgs.length; i++) {
    const ch = rawArgs[i];
    if (inSingle) {
      buf += ch;
      if (ch === "'" && rawArgs[i - 1] !== '\\') inSingle = false;
    } else if (inDouble) {
      buf += ch;
      if (ch === '"' && rawArgs[i - 1] !== '\\') inDouble = false;
    } else if (ch === "'") { inSingle = true; buf += ch; }
    else if (ch === '"') { inDouble = true; buf += ch; }
    else if (ch === '(' || ch === '[' || ch === '{') { depth++; buf += ch; }
    else if (ch === ')' || ch === ']' || ch === '}') { depth--; buf += ch; }
    else if (ch === ',' && depth === 0) { params.push(buf); buf = ''; }
    else buf += ch;
  }
  if (buf.trim()) params.push(buf);
  return params.map((p) => {
    const text = p.trim();
    const eq = text.indexOf('=');
    if (eq === -1) return { name: text, required: true, default: null };
    return {
      name: text.slice(0, eq).trim(),
      required: false,
      default: text.slice(eq + 1).trim(),
    };
  });
}

/**
 * Combine raw extracted macros with their parsed docs + parameter lists into
 * the final manifest entries. Source order is preserved.
 */
export function buildManifest(macros) {
  return macros.map((m) => {
    const parsed = parseDoc(m.doc);
    const params = parseParams(m.rawArgs);
    // Merge per-arg descriptions from the `Args:` block onto the param list.
    const argDescriptions = new Map(parsed.args.map((a) => [a.name, a.description]));
    const args = params.map((p) => ({
      name: p.name,
      required: p.required,
      default: p.default,
      description: argDescriptions.get(p.name) || null,
    }));
    return {
      name: m.name,
      signature: parsed.signature || `${m.name}(${m.rawArgs})`,
      summary: parsed.summary,
      description: parsed.description,
      args,
      example: parsed.example,
      source: m.source,
      documented: m.doc != null,
    };
  });
}
