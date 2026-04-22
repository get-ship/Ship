#!/usr/bin/env node
/**
 * lint-tokens — guard Ship's design-token discipline.
 *
 * Checks (see task spec):
 *  1. Drift between tokens.json and tokens.css (both directions).
 *  2. Hardcoded color literals in component CSS template strings
 *     (rgba/oklch/hex).
 *  3. var(--X) references naming tokens that aren't declared in
 *     tokens.json (typos, drift). Allows component-local --comp-*
 *     scoped vars where <comp> is the file's ship-<name> prefix.
 *  4. color-mix(in oklch, var(--<tone-base>) N%, transparent) formulae
 *     that match the ramp percents (14/22/30%) — these should read
 *     --tone-<name>-{bg,ring,border}.
 *
 * Limitations (regex-based):
 *  - Only scans contents inside `css\`…\`` tagged template blocks.
 *    Block boundaries are detected by counting unescaped backticks
 *    with a depth counter — works for Lit's css`…` blocks which don't
 *    nest another css`…` inside.
 *  - Does not parse CSS; we only look for the specific patterns.
 *  - File paths are absolute in output for terminal click-through.
 *
 * Exit codes:
 *   0 = no errors (warnings allowed)
 *   1 = at least one error
 */
import { readFileSync, readdirSync, statSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { dirname, join, resolve, relative } from 'node:path';

const HERE = dirname(fileURLToPath(import.meta.url));
const PKG = resolve(HERE, '..');
const ROOT = resolve(PKG, '../..');
const TOKENS_JSON = join(PKG, 'tokens.json');
const TOKENS_CSS  = join(PKG, 'src/tokens.css');
const THEMES_CSS  = join(PKG, 'src/themes.css');
const COMPONENTS  = join(PKG, 'src/web-components');

const RED = '\x1b[31m'; const YEL = '\x1b[33m'; const DIM = '\x1b[2m'; const RST = '\x1b[0m';

const errors = [];
const warnings = [];
const push = (arr, f, line, col, msg) => arr.push({ file: f, line, col, msg });

// ── Load JSON schema ──────────────────────────────────────────────
const schema = JSON.parse(readFileSync(TOKENS_JSON, 'utf8'));
const declared = new Set(Object.keys(schema.tokens));

// ── Parse tokens.css for declared properties (at top level of rules) ─
const cssText = readFileSync(TOKENS_CSS, 'utf8');
// Match `--name: value;` occurrences. Drift check only cares about names.
const cssDeclared = new Set();
for (const m of cssText.matchAll(/(--[a-z0-9-]+)\s*:/gi)) {
  cssDeclared.add(m[1]);
}

// Drift: in JSON but not CSS (skip tokens marked declared-in another file).
for (const name of declared) {
  const meta = schema.tokens[name];
  if (meta['declared-in']) continue;
  if (!cssDeclared.has(name)) {
    push(errors, TOKENS_JSON, 0, 0, `tokens.json declares ${name} but tokens.css never defines it`);
  }
}
// Drift: in CSS but not JSON
for (const name of cssDeclared) {
  if (!declared.has(name)) {
    push(errors, TOKENS_CSS, 0, 0, `tokens.css defines ${name} but tokens.json has no entry`);
  }
}

// ── Walk component sources ──────────────────────────────────────────
function walk(dir, out = []) {
  for (const entry of readdirSync(dir)) {
    const p = join(dir, entry);
    const s = statSync(p);
    if (s.isDirectory()) walk(p, out);
    else if (entry.endsWith('.ts') && !entry.endsWith('.test.ts')) out.push(p);
  }
  return out;
}
const files = walk(COMPONENTS);

// Extract ship-<prefix> component prefix from filename for local-var allowlist.
// e.g. ship-dialog.ts / ship-dialog.styles.ts → "dialog" and commonly abbreviated "dlg".
// We accept any --<file-basename-word>-* or --<file-prefix>-* form for local vars;
// a simple heuristic is: any CSS custom property whose base matches the file's
// first-word prefix. We're permissive here to avoid false positives.
function filePrefixes(file) {
  const base = file.split('/').pop().replace(/\.(styles\.)?ts$/, '');
  // "ship-dialog" → ["ship-dialog", "dialog", "dlg"]; abbreviations can't be auto-derived,
  // so we treat ANY --X that's consistently local-looking (contains the component name
  // OR starts with the full file base) as local. The safer check is the base itself.
  const parts = [base];
  const short = base.replace(/^ship-/, '');
  parts.push(short);
  return parts;
}

// Compute posLoc: line/col for a regex index.
function posLoc(src, idx) {
  let line = 1, col = 1;
  for (let i = 0; i < idx; i++) { if (src.charCodeAt(i) === 10) { line++; col = 1; } else col++; }
  return { line, col };
}

// Pull out css`…` template literal bodies with their offsets.
function cssBlocks(src) {
  const blocks = [];
  const re = /\bcss`/g;
  let m;
  while ((m = re.exec(src))) {
    const start = m.index + m[0].length;
    // find matching backtick, skipping escaped ones and ${…} interpolations.
    let i = start; let depth = 0;
    while (i < src.length) {
      const ch = src[i];
      if (ch === '\\') { i += 2; continue; }
      if (ch === '$' && src[i + 1] === '{') { depth++; i += 2; continue; }
      if (ch === '}' && depth > 0) { depth--; i++; continue; }
      if (ch === '`' && depth === 0) break;
      i++;
    }
    blocks.push({ body: src.slice(start, i), offset: start });
    re.lastIndex = i + 1;
  }
  return blocks;
}

// Component-local vars declared or set anywhere in the library. Scanning
// every .ts file once catches cases where the declaration and consumption
// live in sibling files (e.g. ship-tabs.ts calls setProperty('--underline-w')
// on the host and ship-tabs.styles.ts reads `var(--underline-w)`).
const componentLocalVars = new Set();
for (const file of files) {
  const src = readFileSync(file, 'utf8');
  // `--X:` declarations inside css blocks or anywhere in the file.
  for (const m of src.matchAll(/(--[a-z0-9-]+)\s*:/gi)) componentLocalVars.add(m[1]);
  // `setProperty('--X', ...)` — runtime-assigned local vars.
  for (const m of src.matchAll(/setProperty\(\s*['"`](--[a-z0-9-]+)/gi)) componentLocalVars.add(m[1]);
  // `--X` appearing as an exposed component API key in docs/comments.
  for (const m of src.matchAll(/@cssprop\s+(--[a-z0-9-]+)/gi)) componentLocalVars.add(m[1]);
}

// Ramp percent → replacement suffix
const RAMP = { '14%': 'bg', '22%': 'ring', '30%': 'border' };
// Tone bases recognized by the ramp
const TONE_BASES = new Set(['info', 'success', 'warning', 'destructive', 'accent', 'primary']);

for (const file of files) {
  const src = readFileSync(file, 'utf8');
  const prefixes = filePrefixes(file);
  const blocks = cssBlocks(src);
  // Component-local vars: any --X declared in THIS file's css blocks.
  // This captures the "scratch variable" pattern (--btn-*, --dlg-*, --cb-*,
  // --toast-edge, etc.) without needing a rigid prefix rule.
  const localVars = new Set();
  for (const { body } of blocks) {
    for (const m of body.matchAll(/(--[a-z0-9-]+)\s*:/gi)) localVars.add(m[1]);
  }
  for (const { body, offset } of blocks) {
    // 2a. rgba(...) literal
    for (const m of body.matchAll(/\brgba?\s*\(/g)) {
      const { line, col } = posLoc(src, offset + m.index);
      push(errors, file, line, col, `hardcoded ${m[0].trim().replace(/\s*\($/, '')} literal in component CSS — use a token`);
    }
    // 2b. raw oklch(...) literal
    for (const m of body.matchAll(/\boklch\s*\(/g)) {
      const { line, col } = posLoc(src, offset + m.index);
      push(errors, file, line, col, `hardcoded oklch(...) literal in component CSS — use a token (e.g. --backdrop, --tone-*-bg, --shadow-*)`);
    }
    // 2c. hex literal
    for (const m of body.matchAll(/#[0-9a-fA-F]{3,8}\b/g)) {
      // Skip fragment identifiers inside url() or strings? Our regex is inside css blocks only.
      const { line, col } = posLoc(src, offset + m.index);
      push(errors, file, line, col, `hardcoded hex literal ${m[0]} — use a token`);
    }
    // 3. var(--X) not declared
    for (const m of body.matchAll(/var\(\s*(--[a-z0-9-]+)/gi)) {
      const name = m[1];
      if (declared.has(name)) continue;
      if (localVars.has(name)) continue; // declared inside this file's css
      if (componentLocalVars.has(name)) continue; // declared/set anywhere in the library
      // allowlist: --<file-prefix>-*  (local scoped vars by naming convention)
      const local = prefixes.some(p => name.startsWith(`--${p}-`));
      if (local) continue;
      const { line, col } = posLoc(src, offset + m.index);
      push(errors, file, line, col, `var(${name}) is not declared in tokens.json — typo or missing token`);
    }
    // 4. color-mix ramp-percent formulae
    const cmRe = /color-mix\(\s*in\s+oklch\s*,\s*var\((--[a-z0-9-]+)\)\s*(\d+%)\s*,\s*transparent\s*\)/gi;
    for (const m of body.matchAll(cmRe)) {
      const base = m[1].replace(/^--/, '');
      const pct = m[2];
      if (TONE_BASES.has(base) && RAMP[pct]) {
        const suggested = `--tone-${base}-${RAMP[pct]}`;
        const { line, col } = posLoc(src, offset + m.index);
        push(warnings, file, line, col, `ramp-percent color-mix — replace with var(${suggested})`);
      }
    }
  }
}

// ── Report ──────────────────────────────────────────────────────────
function fmt(list, colour, label) {
  if (!list.length) return;
  console.log(`\n${colour}${label} (${list.length})${RST}`);
  for (const { file, line, col, msg } of list) {
    const rel = relative(ROOT, file) || file;
    const loc = line ? `:${line}:${col}` : '';
    console.log(`  ${rel}${loc} ${DIM}—${RST} ${msg}`);
  }
}
fmt(errors, RED, 'errors');
fmt(warnings, YEL, 'warnings');
console.log(`\n${errors.length} error(s), ${warnings.length} warning(s)`);
// silence unused import warning
void THEMES_CSS;
process.exit(errors.length ? 1 : 0);
